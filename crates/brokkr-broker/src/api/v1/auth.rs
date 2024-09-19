use axum::{extract::Query, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PakQuery {
    pak: String,
}

#[derive(Serialize)]
struct AuthResponse {
    is_valid: bool,
    permissions: Option<Permissions>,
}

#[derive(Serialize)]
enum Permissions {
    Admin,
    Agent { uuid: String },
}

pub fn routes() -> Router {
    Router::new().route("/auth/verify-pak", post(verify_pak))
}

async fn verify_pak(Query(params): Query<PakQuery>) -> Json<AuthResponse> {
    // In a real implementation, you would verify the PAK against your database
    // and retrieve the associated permissions. This is a mock implementation.
    let (is_valid, permissions) = match params.pak.as_str() {
        "admin_pak_123" => (true, Some(Permissions::Admin)),
        "agent_pak_456" => (
            true,
            Some(Permissions::Agent {
                uuid: "agent-123".to_string(),
            }),
        ),
        _ => (false, None),
    };

    Json(AuthResponse {
        is_valid,
        permissions,
    })
}
