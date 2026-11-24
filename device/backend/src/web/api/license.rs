use axum::{
    Json,
    body::Bytes,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const DEFAULT_ROOT: &str = "/data/opt/tm-lic";
const LICENSE_PATH: &str = "/data/state/tm-lic/licenses/TMStreamer-LICENSE.json";
const FEATURE_CACHE_TTL: Duration = Duration::from_secs(3);

static FEATURE_CACHE: OnceLock<Mutex<Option<(Instant, LicenseStatus)>>> = OnceLock::new();

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LicenseStatus {
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub license_system: String,
    #[serde(default)]
    pub activated: bool,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub license_id: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub key_id: Option<String>,
}

impl LicenseStatus {
    fn unverified(message: impl Into<String>) -> Self {
        Self {
            status: "UNVERIFIED".into(),
            message: message.into(),
            ..Self::default()
        }
    }

    fn allows(&self, feature: &str) -> bool {
        self.product == "TMSTREAMER"
            && self.license_system == "TM-LIC"
            && self.activated
            && self.status == "LICENSED"
            && self.features.iter().any(|item| item.as_str() == feature)
    }
}

/// Runs the authoritative TM-LIC verifier. The device service runs as root,
/// which is required to read the protected machine identity and key registry.
fn verify_at(license_path: Option<&Path>) -> LicenseStatus {
    let root = std::env::var("TM_LIC_ROOT").unwrap_or_else(|_| DEFAULT_ROOT.into());
    let mut command = Command::new(
        std::env::var("TM_LIC_PYTHON").unwrap_or_else(|_| "/usr/bin/python3".into()),
    );
    command.arg("-m").arg("app.cli").arg("status");
    // Use the same authoritative file for status checks and activation.
    // Do not depend on a CLI default that may point at a different location.
    command
        .arg("--license")
        .arg(license_path.unwrap_or_else(|| Path::new(LICENSE_PATH)));
    let output = command.env("PYTHONPATH", &root).output();

    let output = match output {
        Ok(output) => output,
        Err(error) => return LicenseStatus::unverified(format!("TM-LIC verifier unavailable: {error}")),
    };

    let parsed = serde_json::from_slice::<LicenseStatus>(&output.stdout);
    let status = match parsed {
        Ok(status) => status,
        Err(error) => {
            let detail = String::from_utf8_lossy(&output.stderr);
            return LicenseStatus::unverified(format!(
                "TM-LIC returned an invalid status response: {error}; {detail}"
            ));
        }
    };

    if !output.status.success() || status.product != "TMSTREAMER"
        || status.license_system != "TM-LIC" || !status.activated
        || status.status != "LICENSED"
    {
        return LicenseStatus {
            activated: false,
            status: if status.status.is_empty() { "UNLICENSED".into() } else { status.status.clone() },
            features: Vec::new(),
            message: if status.message.is_empty() {
                "Lisensi TM-LIC belum aktif atau tidak dapat diverifikasi.".into()
            } else {
                status.message.clone()
            },
            ..status
        };
    }
    status
}

pub async fn status() -> Json<LicenseStatus> {
    Json(tokio::task::spawn_blocking(|| verify_at(None)).await.unwrap_or_else(|error| {
        LicenseStatus::unverified(format!("TM-LIC verification task failed: {error}"))
    }))
}

async fn checked_status() -> LicenseStatus {
    tokio::task::spawn_blocking(|| verify_at(None)).await.unwrap_or_else(|error| {
        LicenseStatus::unverified(format!("TM-LIC verification task failed: {error}"))
    })
}

/// A short cache keeps per-frame fan-out checks cheap while making expiry or
/// revocation effective within a few seconds. HTTP middleware still performs
/// a fresh authoritative check for every protected request.
pub fn feature_enabled(feature: &str) -> bool {
    let cache = FEATURE_CACHE.get_or_init(|| Mutex::new(None));
    let mut cached = cache.lock().unwrap();
    if let Some((checked, status)) = cached.as_ref()
        && checked.elapsed() < FEATURE_CACHE_TTL
    {
        return status.allows(feature);
    }
    let status = verify_at(None);
    let allowed = status.allows(feature);
    *cached = Some((Instant::now(), status));
    allowed
}

fn invalidate_feature_cache() {
    if let Some(cache) = FEATURE_CACHE.get() {
        *cache.lock().unwrap() = None;
    }
}

async fn require(feature: &'static str, req: Request, next: Next) -> Response {
    let status = checked_status().await;
    if status.allows(feature) {
        return next.run(req).await;
    }

    let message = if status.message.is_empty() {
        format!("Fitur {feature} memerlukan lisensi TM-LIC yang valid.")
    } else {
        status.message.clone()
    };
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": message,
            "license": status,
            "required_feature": feature
        })),
    )
        .into_response()
}

pub async fn streaming_only(req: Request, next: Next) -> Response {
    require("streaming", req, next).await
}

pub async fn recording_only(req: Request, next: Next) -> Response {
    require("recording", req, next).await
}

/// Accepts only a signed, machine-bound TM-LIC license document. The verifier
/// validates the temporary file before it can replace the installed license.
pub async fn activate(
    State(services): State<super::Services>,
    body: Bytes,
) -> Result<Json<LicenseStatus>, Response> {
    if body.is_empty() || body.len() > 64 * 1024 {
        return Err((StatusCode::BAD_REQUEST, "License file must be between 1 byte and 64 KiB").into_response());
    }
    let result = tokio::task::spawn_blocking(move || -> Result<LicenseStatus, String> {
        let final_path = PathBuf::from("/data/state/tm-lic/licenses/TMStreamer-LICENSE.json");
        let parent = final_path.parent().ok_or("Invalid TM-LIC license path")?;
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot prepare TM-LIC license directory: {e}"))?;
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("Cannot protect TM-LIC license directory: {e}"))?;
        let temp_path = parent.join(format!(".TMStreamer-LICENSE.{}.tmp", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .write(true).create_new(true).mode(0o600).open(&temp_path)
            .map_err(|e| format!("Cannot stage license file: {e}"))?;
        if let Err(error) = file.write_all(&body).and_then(|_| file.sync_all()) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(format!("Cannot write license file: {error}"));
        }
        drop(file);

        let verified = verify_at(Some(&temp_path));
        let valid = verified.product == "TMSTREAMER"
            && verified.license_system == "TM-LIC"
            && verified.activated
            && verified.status == "LICENSED";
        if !valid {
            let _ = std::fs::remove_file(&temp_path);
            return Err(if verified.message.is_empty() {
                "License signature or device binding could not be verified.".into()
            } else {
                verified.message
            });
        }

        std::fs::rename(&temp_path, &final_path)
            .map_err(|e| format!("Cannot install verified TM-LIC license: {e}"))?;
        if let Ok(directory) = std::fs::File::open(parent) {
            let _ = directory.sync_all();
        }
        invalidate_feature_cache();

        // The service may have started while this device was unlicensed, in
        // which case paid output transports were deliberately left stopped.
        // Re-apply only the entitlements present in the newly verified file.
        let cfg = services.store.get();
        if verified.allows("streaming") {
            if let Err(error) = services.rtsp.apply(&cfg.rtsp) {
                eprintln!("[license] RTSP could not be enabled after activation: {error}");
            }
            if let Err(error) = services.srt.apply(&cfg.srt) {
                eprintln!("[license] SRT could not be enabled after activation: {error}");
            }
            services.rtmp.apply(&cfg.rtmp);
            if let Err(error) = services.udp.apply(&cfg.udp) {
                eprintln!("[license] UDP could not be enabled after activation: {error}");
            }
        }
        if verified.allows("recording") {
            services.recorder.configure(&cfg.recording);
            services.button.configure(&cfg.recording.button);
        }
        Ok(verified)
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("TM-LIC activation failed: {e}")).into_response())?;

    result.map(Json).map_err(|message| (StatusCode::UNPROCESSABLE_ENTITY, message).into_response())
}

/// Protect settings that can enable paid output functions. Hardware, network,
/// receiver, display and HDMI setup remain available without a license.
pub async fn settings_only(req: Request, next: Next) -> Response {
    let path = req.uri().path();
    let required: &[&str] = if (path == "/settings" && req.method() == axum::http::Method::PUT)
        || path == "/settings/reset"
    {
        &["streaming", "recording"]
    } else if let Some(section) = path.strip_prefix("/settings/") {
        match section {
            "rtsp" | "srt" | "rtmp" | "udp" | "viewer" => &["streaming"],
            "recording" => &["recording"],
            _ => &[],
        }
    } else {
        &[]
    };

    if required.is_empty() {
        return next.run(req).await;
    }
    let status = checked_status().await;
    if let Some(feature) = required.iter().find(|feature| !status.allows(feature)) {
        let message = if status.message.is_empty() {
            format!("Fitur {feature} memerlukan lisensi TM-LIC yang valid.")
        } else {
            status.message.clone()
        };
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": message,
                "license": status,
                "required_feature": feature
            })),
        ).into_response();
    }
    next.run(req).await
}


#[cfg(test)]
mod tests {
    use super::*;

    fn licensed(features: &[&str]) -> LicenseStatus {
        LicenseStatus {
            product: "TMSTREAMER".into(),
            license_system: "TM-LIC".into(),
            activated: true,
            status: "LICENSED".into(),
            features: features.iter().map(|feature| (*feature).into()).collect(),
            ..LicenseStatus::default()
        }
    }

    #[test]
    fn grants_only_explicit_tm_lic_entitlements() {
        let status = licensed(&["streaming", "recording"]);
        assert!(status.allows("streaming"));
        assert!(status.allows("recording"));
        assert!(!status.allows("camera"));
        assert!(!status.allows("admin"));
    }

    #[test]
    fn rejects_wrong_product_license_system_and_inactive_status() {
        let mut status = licensed(&["streaming", "recording"]);
        status.product = "OTHER".into();
        assert!(!status.allows("streaming"));

        let mut status = licensed(&["streaming"]);
        status.license_system = "OTHER".into();
        assert!(!status.allows("streaming"));

        let mut status = licensed(&["streaming"]);
        status.activated = false;
        assert!(!status.allows("streaming"));

        let mut status = licensed(&["streaming"]);
        status.status = "EXPIRED".into();
        assert!(!status.allows("streaming"));
    }

    #[test]
    fn missing_entitlement_fails_closed() {
        let status = licensed(&["streaming"]);
        assert!(status.allows("streaming"));
        assert!(!status.allows("recording"));
        assert!(!LicenseStatus::unverified("verifier unavailable").allows("streaming"));
    }
}
