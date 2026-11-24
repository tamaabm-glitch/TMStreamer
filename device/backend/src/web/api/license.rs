use axum::{
    Json,
    body::Bytes,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{io::Write, os::unix::fs::OpenOptionsExt, path::{Path, PathBuf}, process::Command};

const DEFAULT_ROOT: &str = "/data/opt/tm-lic";

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
            && self.features.iter().any(|item| item == feature)
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
    if let Some(path) = license_path {
        command.arg("--license").arg(path);
    }
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

pub fn feature_enabled(feature: &str) -> bool {
    verify_at(None).allows(feature)
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
pub async fn activate(body: Bytes) -> Result<Json<LicenseStatus>, Response> {
    if body.is_empty() || body.len() > 64 * 1024 {
        return Err((StatusCode::BAD_REQUEST, "License file must be between 1 byte and 64 KiB").into_response());
    }
    let result = tokio::task::spawn_blocking(move || -> Result<LicenseStatus, String> {
        let final_path = PathBuf::from("/data/state/tm-lic/licenses/TMStreamer-LICENSE.json");
        let parent = final_path.parent().ok_or("Invalid TM-LIC license path")?;
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot prepare TM-LIC license directory: {e}"))?;
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
        Ok(verified)
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("TM-LIC activation failed: {e}")).into_response())?;

    result.map(Json).map_err(|message| (StatusCode::UNPROCESSABLE_ENTITY, message).into_response())
}

/// Protect settings that can enable paid output functions. Hardware, network,
/// receiver, display and HDMI setup remain available without a license.
pub async fn settings_only(req: Request, next: Next) -> Response {
    let path = req.uri().path();
    let required: &[&str] = if path == "/settings" && req.method() == axum::http::Method::PUT {
        &["streaming", "recording"]
    } else if path == "/settings/reset" {
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
