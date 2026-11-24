use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
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

fn required_features(path: &str, method: &axum::http::Method) -> &'static [&'static str] {
    let recording_settings = path == "/settings/recording" && *method == axum::http::Method::PUT;
    let streaming_settings = matches!(
        path,
        "/settings/rtsp"
            | "/settings/srt"
            | "/settings/rtmp"
            | "/settings/udp"
            | "/settings/viewer"
    ) && *method == axum::http::Method::PUT;

    if path.starts_with("/recording") || recording_settings {
        &["recording"]
    } else if streaming_settings || matches!(path, "/offer" | "/video") {
        &["streaming"]
    } else if path == "/settings" && *method == axum::http::Method::PUT {
        &["recording", "streaming"]
    } else {
        &[]
    }
}

pub(crate) async fn enforce(
    axum::extract::State(license): axum::extract::State<crate::tm_lic::LicenseService>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let path = path.strip_prefix("/api").unwrap_or(path);
    let required_features = required_features(path, request.method());

    for feature in required_features {
        if !license.has_feature(feature) {
            let (message, code) = if license.status().licensed {
                (
                    "Fitur ini tidak termasuk dalam lisensi aktif",
                    "feature_not_licensed",
                )
            } else {
                (
                    "Aktifkan lisensi TMStreamer untuk menggunakan fitur ini",
                    "license_required",
                )
            };
            return (
                StatusCode::FORBIDDEN,
                axum::Json(LicenseRequired {
                    error: message,
                    code,
                }),
            )
                .into_response();
        }
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::required_features;
    use axum::http::Method;

    #[test]
    fn gates_all_streaming_websocket_routes() {
        for path in ["/offer", "/video"] {
            assert_eq!(required_features(path, &Method::GET), &["streaming"]);
        }
    }

    #[test]
    fn gates_recording_routes_and_settings() {
        assert_eq!(required_features("/recording/start", &Method::POST), &["recording"]);
        assert_eq!(required_features("/settings/recording", &Method::PUT), &["recording"]);
    }

    #[test]
    fn gates_streaming_settings_and_full_settings_replacement() {
        assert_eq!(required_features("/settings/srt", &Method::PUT), &["streaming"]);
        assert_eq!(required_features("/settings", &Method::PUT), &["recording", "streaming"]);
    }

    #[test]
    fn keeps_device_setup_and_hdmi_available_without_license() {
        for path in ["/auth", "/license", "/network", "/input", "/display", "/system", "/system/logs"] {
            assert!(required_features(path, &Method::GET).is_empty(), "{path}");
        }
    }

    #[test]
    fn keeps_dji_control_and_telemetry_channel_available() {
        assert!(required_features("/control", &Method::GET).is_empty());
    }

    #[test]
    fn does_not_gate_streaming_settings_reads() {
        assert!(required_features("/settings/srt", &Method::GET).is_empty());
    }
}
