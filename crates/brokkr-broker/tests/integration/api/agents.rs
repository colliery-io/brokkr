use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
};
use brokkr_models::models::agents::Agent;

use tower::ServiceExt;

// Import the TestFixture
use crate::fixtures::TestFixture;
