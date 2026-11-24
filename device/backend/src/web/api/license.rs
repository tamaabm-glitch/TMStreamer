use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::tm_lic::LicenseStatus;
use crate::web::auth::Access;

use super::{ApiError, Services, bad};

pub(super) async fn status(State(services): State<Services>) -> Json<LicenseStatus> {
    Json(services.license.status())
}

pub(super) async fn activate(
    State(services): State<Services>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<LicenseStatus>, ApiError> {
    if services.license.status().licensed
        && services.auth.enabled()
        && services.auth.access(&headers) != Access::Admin
    {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Login administrator diperlukan untuk mengganti lisensi aktif".into(),
        ));
    }
    let status = services.license.activate(&body).map_err(bad)?;
    services.apply_entitlements();
    Ok(Json(status))
}

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
struct LicenseRequired {
    error: &'static str,
    code: &'static str,
}

pub(crate) async fn enforce(
    axum::extract::State(license): axum::extract::State<crate::tm_lic::LicenseService>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let path = path.strip_prefix("/api").unwrap_or(path);
    let method = request.method();

    let required_features: &[&str] = if path.starts_with("/recording") {
        &["recording"]
    } else if path == "/settings/recording" && *method == axum::http::Method::PUT {
        &["recording"]
    } else if matches!(path,
        "/settings/rtsp" | "/settings/srt" | "/settings/rtmp"
        | "/settings/udp" | "/settings/viewer"
    ) && *method == axum::http::Method::PUT {
        &["streaming"]
    } else if matches!(path, "/offer" | "/video") {
        &["streaming"]
    } else if path == "/settings" && *method == axum::http::Method::PUT {
        &["recording", "streaming"]
    } else {
        &[]
    };

    for feature in required_features {
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
