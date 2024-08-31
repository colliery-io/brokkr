pub mod stacks;


use axum::Router;
use crate::dal::DAL;



pub fn configure_routes() -> Router<DAL> {
    Router::new()
        .merge(stacks::configure_routes())
}