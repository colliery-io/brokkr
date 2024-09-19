// pub mod stacks;
// pub mod agents;
// pub mod deployment_objects;

use crate::dal::DAL;
use axum::Router;

pub fn configure_routes() -> Router<DAL> {
    Router::new()
    // .merge(stacks::configure_routes())
    // .merge(agents::configure_routes())
    // .merge(deployment_objects::configure_routes())
}
