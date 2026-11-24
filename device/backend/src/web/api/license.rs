use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::process::Command;

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
fn verify() -> LicenseStatus {
    let root = std::env::var("TM_LIC_ROOT").unwrap_or_else(|_| DEFAULT_ROOT.into());
    let output = Command::new(
        std::env::var("TM_LIC_PYTHON").unwrap_or_else(|_| "/usr/bin/python3".into()),
    )
    .arg("-m")
    .arg("app.cli")
    .arg("status")
    .env("PYTHONPATH", &root)
    .output();

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
            status: if status.status.is_empty() { "UNLICENSED".into() } else { status.status },
            features: Vec::new(),
            message: if status.message.is_empty() {
                "Lisensi TM-LIC belum aktif atau tidak dapat diverifikasi.".into()
            } else {
                status.message
            },
            ..status
        };
    }
    status
}

pub async fn status() -> Json<LicenseStatus> {
    Json(tokio::task::spawn_blocking(verify).await.unwrap_or_else(|error| {
        LicenseStatus::unverified(format!("TM-LIC verification task failed: {error}"))
    }))
}

async fn require(feature: &'static str, req: Request, next: Next) -> Response {
    let status = tokio::task::spawn_blocking(verify).await.unwrap_or_else(|error| {
        LicenseStatus::unverified(format!("TM-LIC verification task failed: {error}"))
    });
    if status.allows(feature) {
        return next.run(req).await;
    }

    let message = if status.message.is_empty() {
        format!("Fitur {feature} memerlukan lisensi TM-LIC yang valid.")
    } else {
        status.message
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
