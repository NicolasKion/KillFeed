//! The setup page answers.

use crate::common::TestApp;

#[tokio::test]
async fn it_returns_a_successful_response() {
    let app = TestApp::new().await;

    assert_eq!(app.get("/api/pages/home").await.status, 200);
}
