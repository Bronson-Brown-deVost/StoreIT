use crate::common::{AdminTestApp, TestApp};

#[tokio::test]
async fn local_login_valid_credentials_returns_session_cookie() {
    let app = AdminTestApp::spawn().await;

    let bare_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = bare_client
        .post(app.url("/api/v1/auth/local/login"))
        .json(&serde_json::json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    // Verify session cookie was set
    let set_cookie_headers: Vec<_> = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();

    let has_session = set_cookie_headers
        .iter()
        .any(|c| c.starts_with("storeit_session="));
    assert!(
        has_session,
        "session cookie should be set after local login"
    );

    // Verify the response body contains user info
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["user"]["email"], "admin@test.com");
    assert_eq!(body["user"]["display_name"], "Test Admin");
    assert!(body["active_group_id"].is_string());
    assert!(!body["groups"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn local_login_wrong_password_returns_401() {
    let app = AdminTestApp::spawn().await;

    let bare_client = reqwest::Client::new();

    let res = bare_client
        .post(app.url("/api/v1/auth/local/login"))
        .json(&serde_json::json!({
            "username": "admin",
            "password": "wrong-password"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn local_login_nonexistent_user_returns_401() {
    let app = AdminTestApp::spawn().await;

    let bare_client = reqwest::Client::new();

    let res = bare_client
        .post(app.url("/api/v1/auth/local/login"))
        .json(&serde_json::json!({
            "username": "no-such-user",
            "password": "any-password"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn local_login_when_oidc_mode_user_not_found() {
    // TestApp uses AuthMode::Oidc — no local users are seeded, so login
    // should fail with 401 (the user won't exist in the DB).
    let app = TestApp::spawn().await;

    let bare_client = reqwest::Client::new();

    let res = bare_client
        .post(app.url("/api/v1/auth/local/login"))
        .json(&serde_json::json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await
        .unwrap();

    // The local login endpoint itself doesn't gate on auth_mode; it simply
    // won't find any local user, so it returns 401.
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn local_login_non_admin_user_succeeds() {
    let app = AdminTestApp::spawn().await;

    let bare_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = bare_client
        .post(app.url("/api/v1/auth/local/login"))
        .json(&serde_json::json!({
            "username": "user",
            "password": "user123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["user"]["email"], "user@test.com");
    assert_eq!(body["user"]["display_name"], "Test User");
}

#[tokio::test]
async fn local_login_session_cookie_works_for_me() {
    let app = AdminTestApp::spawn().await;

    // Login with cookie store so we can reuse the session
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    let login_res = client
        .post(app.url("/api/v1/auth/local/login"))
        .json(&serde_json::json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_res.status(), 200);

    // Use the session to call /me
    let me_res = client.get(app.url("/api/v1/auth/me")).send().await.unwrap();
    assert_eq!(me_res.status(), 200);

    let body: serde_json::Value = me_res.json().await.unwrap();
    assert_eq!(body["user"]["email"], "admin@test.com");
}
