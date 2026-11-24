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
