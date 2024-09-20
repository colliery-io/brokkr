use axum::{routing::post, Json, Router};
use crate::api::v1::middleware::AuthPayload;
use axum::extract::Extension;
use crate::api::v1::middleware::AuthResponse;
use crate::dal::DAL;
pub fn routes() -> Router<DAL> {
    Router::new().route("/auth/pak", post(verify_pak))
}



async fn verify_pak(Extension(auth_payload): Extension<AuthPayload>) -> Json<AuthResponse> {
    Json(AuthResponse {
        admin: auth_payload.admin,
        agent: auth_payload.agent.map(|id| id.to_string()),
        generator: auth_payload.generator.map(|id| id.to_string()),
    })
}
