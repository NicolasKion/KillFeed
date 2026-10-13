//! tests/Feature/UserAgentTest.php: every outgoing request names the app
//! and its maintainer.

use crate::common::TestApp;

#[tokio::test]
async fn outgoing_requests_identify_the_app_and_its_maintainer() {
    let app = TestApp::new().await;

    let _ = app.state.eve.esi_get("/status").await;
    let _ = app.state.eve.zkillboard_get("/api/characterID/1/").await;
    let _ = app.state.eve.r2z2_get("/sequence.json").await;

    let received = app.eve.received();
    assert_eq!(received.len(), 3);
    for request in received {
        let user_agent = request.header("user-agent").unwrap_or_default();
        assert!(user_agent.contains("KillFeed"), "{}", request.path);
        assert!(
            user_agent.contains("nicolaskion07@gmail.com"),
            "{}",
            request.path
        );
        assert!(user_agent.contains("Nicolas Kion"), "{}", request.path);
    }
}
