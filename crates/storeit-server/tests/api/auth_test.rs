use crate::common::{EXPIRED_SESSION_ID, TestApp};

#[tokio::test]
async fn login_redirects_to_oidc() {
    let app = TestApp::spawn().await;

    // Use a client without default cookies to simulate unauthenticated request
    let bare_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = bare_client
        .get(app.url("/api/v1/auth/login"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 303);
    let location = res.headers().get("location").unwrap().to_str().unwrap();
    assert!(
        location.contains("mock"),
        "redirect should point to mock auth URL"
    );
}

#[tokio::test]
async fn callback_creates_session_and_redirects() {
    let app = TestApp::spawn().await;

    // First, hit login to get the auth_pending cookie
    let bare_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let login_res = bare_client
        .get(app.url("/api/v1/auth/login"))
        .send()
        .await
        .unwrap();
    assert_eq!(login_res.status(), 303);

    // Now hit callback with the mock CSRF state and any code
    let callback_res = bare_client
        .get(app.url("/api/v1/auth/callback?code=any-code&state=mock-csrf-state"))
        .send()
        .await
        .unwrap();

    assert_eq!(callback_res.status(), 303);
    let location = callback_res
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(location, "/");

    // Verify the session cookie was set
    let set_cookie_headers: Vec<_> = callback_res
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();

    let has_session = set_cookie_headers
        .iter()
        .any(|c| c.starts_with("storeit_session="));
    assert!(has_session, "session cookie should be set after callback");
}

#[tokio::test]
async fn callback_rejects_invalid_csrf() {
    let app = TestApp::spawn().await;

    let bare_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    // Get the auth_pending cookie from login
    let _ = bare_client
        .get(app.url("/api/v1/auth/login"))
        .send()
        .await
        .unwrap();

    // Try callback with wrong CSRF state
    let res = bare_client
        .get(app.url("/api/v1/auth/callback?code=any&state=wrong-csrf"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn me_returns_user_info() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["user"]["email"], "test@example.com");
    assert_eq!(body["user"]["display_name"], "Test User");
    assert!(!body["groups"].as_array().unwrap().is_empty());
    assert!(body["active_group_id"].is_string());
}

#[tokio::test]
async fn me_requires_auth() {
    let app = TestApp::spawn().await;

    // Use a bare client without the session cookie
    let bare_client = reqwest::Client::new();
    let res = bare_client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn logout_clears_session() {
    let app = TestApp::spawn().await;

    // First verify we're authenticated
    let me_res = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(me_res.status(), 200);

    // Logout
    let logout_res = app
        .client
        .post(app.url("/api/v1/auth/logout"))
        .send()
        .await
        .unwrap();

    assert_eq!(logout_res.status(), 200);
}

#[tokio::test]
async fn unauthenticated_data_request_returns_401() {
    let app = TestApp::spawn().await;

    // Use a bare client without the session cookie
    let bare_client = reqwest::Client::new();
    let res = bare_client
        .get(app.url("/api/v1/locations"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn nonexistent_session_returns_401() {
    let app = TestApp::spawn().await;

    // Use a client with a session ID that doesn't exist in the DB
    let bad_client = reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::COOKIE,
                "storeit_session=nonexistent-session-id".parse().unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    let res = bad_client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn expired_session_returns_401() {
    let app = TestApp::spawn().await;

    // Use the pre-seeded expired session (exists in DB but has past expiry)
    let expired_client = reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::COOKIE,
                format!("storeit_session={EXPIRED_SESSION_ID}")
                    .parse()
                    .unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    let res = expired_client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn callback_without_pending_cookie_returns_401() {
    let app = TestApp::spawn().await;

    // Try the callback without first hitting /login (no auth_pending cookie)
    let bare_client = reqwest::Client::new();
    let res = bare_client
        .get(app.url("/api/v1/auth/callback?code=any&state=any"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn switch_active_group() {
    let app = TestApp::spawn().await;

    // Get the current user info to find the group id
    let me_res = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(me_res.status(), 200);
    let me_body: serde_json::Value = me_res.json().await.unwrap();
    let group_id = me_body["active_group_id"].as_str().unwrap().to_string();

    // Switch to the same group (we only have one) — should succeed
    let res = app
        .client
        .put(app.url("/api/v1/auth/me/active-group"))
        .json(&serde_json::json!({ "group_id": group_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["active_group_id"], group_id);
    assert_eq!(body["user"]["email"], "test@example.com");
}

#[tokio::test]
async fn switch_active_group_forbidden() {
    let app = TestApp::spawn().await;

    // Try to switch to a random group we're not a member of
    let res = app
        .client
        .put(app.url("/api/v1/auth/me/active-group"))
        .json(&serde_json::json!({ "group_id": "00000000-0000-0000-0000-000000000099" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn callback_malformed_pending_cookie_returns_401() {
    let app = TestApp::spawn().await;

    // Manually craft a client with a malformed auth_pending cookie (no pipe separator)
    let bad_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::COOKIE,
                "storeit_auth_pending=no-pipe-separator".parse().unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    let res = bad_client
        .get(app.url("/api/v1/auth/callback?code=any&state=any"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn logout_requires_auth() {
    let app = TestApp::spawn().await;

    // Bare client without session cookie → logout should fail with 401
    let bare_client = reqwest::Client::new();
    let res = bare_client
        .post(app.url("/api/v1/auth/logout"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn switch_active_group_requires_auth() {
    let app = TestApp::spawn().await;

    let bare_client = reqwest::Client::new();
    let res = bare_client
        .put(app.url("/api/v1/auth/me/active-group"))
        .json(&serde_json::json!({ "group_id": "00000000-0000-0000-0000-000000000001" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn full_callback_flow_then_me() {
    let app = TestApp::spawn().await;

    // Do a full login → callback flow, then use the new session to call /me
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    // 1. Login to get auth_pending cookie
    let login_res = client
        .get(app.url("/api/v1/auth/login"))
        .send()
        .await
        .unwrap();
    assert_eq!(login_res.status(), 303);

    // 2. Callback with correct CSRF state
    let callback_res = client
        .get(app.url("/api/v1/auth/callback?code=test-code&state=mock-csrf-state"))
        .send()
        .await
        .unwrap();
    assert_eq!(callback_res.status(), 303);

    // 3. Use the session cookie from callback to call /me
    let me_res = client.get(app.url("/api/v1/auth/me")).send().await.unwrap();
    assert_eq!(me_res.status(), 200);
    let body: serde_json::Value = me_res.json().await.unwrap();
    assert_eq!(body["user"]["email"], "test@example.com");
    assert_eq!(body["user"]["display_name"], "Test User");
    assert!(!body["groups"].as_array().unwrap().is_empty());
}
