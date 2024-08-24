mod agents;
mod stacks;
mod deployment_objects;
mod agent_events;


use crate::fixtures::TestFixture;


#[tokio::test]
async fn test_api_routes_configuration() {
    let fixture = TestFixture::new(); // Use default DATABASE_URL
    let _app = fixture.create_test_router();

    // Your test cases here...
}