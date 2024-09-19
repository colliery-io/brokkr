use axum::{routing::post, Json, Router};
use serde::Serialize;
use crate::api::v1::middleware::AuthPayload;
use axum::extract::Extension;


pub fn routes() -> Router {
    Router::new().route("/auth/pak", post(verify_pak))
}

#[derive(Serialize)]
struct AuthResponse {
    admin: bool,
    agent: Option<String>,
    generator: Option<String>,
}

async fn verify_pak(Extension(auth_payload): Extension<AuthPayload>) -> Json<AuthResponse> {
    Json(AuthResponse {
        admin: auth_payload.admin,
        agent: auth_payload.agent.map(|id| id.to_string()),
        generator: auth_payload.generator.map(|id| id.to_string()),
    })
}
