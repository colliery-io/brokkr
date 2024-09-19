use axum::{
    body::Body, extract::State, http::{Request, StatusCode}, middleware::Next, response::Response
};
use uuid::Uuid;
use diesel::prelude::*;
use brokkr_models::schema::admin_role;

use crate::dal::DAL;
use crate::utils::pak;

#[derive(Clone, Debug)]
pub struct AuthPayload {
    pub admin: bool,
    pub agent: Option<Uuid>,
    pub generator: Option<Uuid>,
}

pub async fn auth_middleware<B>(
    State(dal): State<DAL>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let pak = request
        .headers()
        .get("X-PAK")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_payload = verify_pak(&dal, pak).await?;

    request.extensions_mut().insert(auth_payload);

    Ok(next.run(request).await)
}

async fn verify_pak(dal: &DAL, pak: &str) -> Result<AuthPayload, StatusCode> {
    

    let conn = &mut dal.pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check admin role
    let admin_key = admin_role::table
        .select(admin_role::pak_hash)
        .first::<String>(conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(admin_hash) = admin_key {
        if pak::verify_pak(pak.to_string(), admin_hash) {
            return Ok(AuthPayload {
                admin: true,
                agent: None,
                generator: None,
            });
        }
    }

    // Check agents
    let agents = dal.agents().list().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for agent in agents {
        if pak::verify_pak(pak.to_string(), agent.pak_hash) {
            return Ok(AuthPayload {
                admin: false,
                agent: Some(agent.id),
                generator: None,
            });
        }
    }

    // Check generators
    let generators = dal.generators().list().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for generator in generators {
        if pak::verify_pak(pak.to_string(), generator.pak_hash.unwrap_or_default()) {
            return Ok(AuthPayload {
                admin: false,
                agent: None,
                generator: Some(generator.id),
            });
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
