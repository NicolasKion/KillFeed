//! tests/Feature/DemoTest.php: the demo page's options.

use crate::common::TestApp;

#[tokio::test]
async fn demo_page_renders_with_defaults() {
    let app = TestApp::new().await;

    let props = app.get("/api/pages/demo").await.ok_json();

    assert_eq!(props["count"], 50);
    assert_eq!(props["hours"], 168);
    assert_eq!(props["summary"], true);
    assert_eq!(props["meta"], true);
}

#[tokio::test]
async fn demo_page_clamps_parameters_to_configured_maximums() {
    let app = TestApp::new().await;

    let props = app
        .get("/api/pages/demo?count=9999&hours=9999&summary=1&meta=0")
        .await
        .ok_json();

    assert_eq!(props["count"], 50);
    assert_eq!(props["hours"], 168);
    assert_eq!(props["summary"], true);
    assert_eq!(props["meta"], false);
}
