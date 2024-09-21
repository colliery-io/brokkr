use crate::dal::DAL;
use crate::api::v1::middleware::AuthPayload;
use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use brokkr_models::models::generator::{Generator, NewGenerator};
use uuid::Uuid;
use crate::utils::pak;
use axum::http::StatusCode;

pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/generators", get(list_generators))
        .route("/generators", post(create_generator))
        .route("/generators/:id", get(get_generator))
        .route("/generators/:id", put(update_generator))
        .route("/generators/:id", delete(delete_generator))
}

async fn list_generators(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Generator>>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.generators().list() {
        Ok(generators) => Ok(Json(generators)),
        Err(e) => {
            eprintln!("Error fetching generators: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch generators"})),
            ))
        }
    }
}

async fn create_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(new_generator): Json<NewGenerator>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    let (pak, pak_hash) = pak::create_pak().map_err(|e| {
        eprintln!("Error creating PAK: {:?}", e);
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create PAK"})),
        )
    })?;

    match dal.generators().create(&new_generator) {
        Ok(generator) => {
            match dal.generators().update_pak_hash(generator.id, pak_hash) {
                Ok(updated_generator) => Ok(Json(serde_json::json!({
                    "generator": updated_generator,
                    "pak": pak
                }))),
                Err(e) => {
                    eprintln!("Error updating generator PAK hash: {:?}", e);
                    Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to update generator PAK hash"})),
                    ))
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating generator: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create generator"})),
            ))
        }
    }
}

async fn get_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Generator>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized access"})),
        ));
    }

    match dal.generators().get(id) {
        Ok(Some(generator)) => Ok(Json(generator)),
        Ok(None) => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Generator not found"})),
        )),
        Err(e) => {
            eprintln!("Error fetching generator: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch generator"})),
            ))
        }
    }
}

async fn update_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(updated_generator): Json<Generator>,
) -> Result<Json<Generator>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized access"})),
        ));
    }

    match dal.generators().update(id, &updated_generator) {
        Ok(generator) => Ok(Json(generator)),
        Err(e) => {
            eprintln!("Error updating generator: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update generator"})),
            ))
        }
    }
}

async fn delete_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized access"})),
        ));
    }

    match dal.generators().soft_delete(id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            eprintln!("Error deleting generator: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to delete generator"})),
            ))
        }
    }
}