use axum::{
    body::Bytes,
    extract::State,
    Json,
};

use crate::tm_lic::LicenseStatus;

use super::{ApiError, Services, bad};

pub(super) async fn status(State(services): State<Services>) -> Json<LicenseStatus> {
    Json(services.license.status())
}

pub(super) async fn activate(
    State(services): State<Services>,
    body: Bytes,
) -> Result<Json<LicenseStatus>, ApiError> {
    services.license.activate(&body)
        .map(Json)
        .map_err(bad)
}

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
struct LicenseRequired {
    error: &'static str,
    code: &'static str,
}

pub(super) async fn enforce(
    axum::extract::State(license): axum::extract::State<crate::tm_lic::LicenseService>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let path = path.strip_prefix("/api").unwrap_or(path);
    let method = request.method();

    let required_feature = if path.starts_with("/recording") {
        Some("recording")
    } else if path == "/settings/recording" && *method == axum::http::Method::PUT {
        Some("recording")
    } else if matches!(path,
        "/settings/rtsp" | "/settings/srt" | "/settings/rtmp"
        | "/settings/udp" | "/settings/viewer"
    ) && *method == axum::http::Method::PUT {
        Some("streaming")
    } else if path == "/settings" && *method == axum::http::Method::PUT {
        Some("streaming")
    } else {
        None
    };

    if let Some(feature) = required_feature {
        if !license.has_feature(feature) {
            let (message, code) = if license.status().licensed {
                ("Fitur ini tidak termasuk dalam lisensi aktif", "feature_not_licensed")
            } else {
                ("Aktifkan lisensi TMStreamer untuk menggunakan fitur ini", "license_required")
            };
            return (
                StatusCode::FORBIDDEN,
                axum::Json(LicenseRequired { error: message, code }),
            ).into_response();
        }
    }

    next.run(request).await
}
