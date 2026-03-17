use crate::common::AdminTestApp;

#[tokio::test]
async fn list_users_returns_seeded_users() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let users = body.as_array().unwrap();

    // AdminTestApp seeds two users: admin and user
    assert!(users.len() >= 2);

    let usernames: Vec<&str> = users
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert!(usernames.contains(&"admin"));
    assert!(usernames.contains(&"user"));
}

#[tokio::test]
async fn list_users_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .non_admin_client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn list_users_unauthenticated_returns_401() {
    let app = AdminTestApp::spawn().await;

    let bare = reqwest::Client::new();
    let res = bare
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn create_user_succeeds() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "newuser",
            "email": "newuser@test.com",
            "display_name": "New User",
            "password": "secret123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["username"], "newuser");
    assert_eq!(body["email"], "newuser@test.com");
    assert_eq!(body["display_name"], "New User");
    assert_eq!(body["is_admin"], false);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn create_admin_user_succeeds() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "adminuser",
            "email": "adminuser@test.com",
            "display_name": "Admin User 2",
            "password": "secret123",
            "is_admin": true
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["is_admin"], true);
}

#[tokio::test]
async fn create_user_empty_username_returns_400() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "",
            "email": "x@test.com",
            "display_name": "X",
            "password": "secret"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn create_user_empty_password_returns_400() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "emptypass",
            "email": "x@test.com",
            "display_name": "X",
            "password": ""
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn create_user_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .non_admin_client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "sneaky",
            "email": "sneaky@test.com",
            "display_name": "Sneaky",
            "password": "secret"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn update_user_changes_email_and_display_name() {
    let app = AdminTestApp::spawn().await;

    // Create a user to update
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "toupdate",
            "email": "old@test.com",
            "display_name": "Old Name",
            "password": "secret"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.unwrap();
    let user_id = created["id"].as_str().unwrap();

    // Update the user
    let res = app
        .client
        .put(app.url(&format!("/api/v1/admin/users/{user_id}")))
        .json(&serde_json::json!({
            "email": "new@test.com",
            "display_name": "New Name"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["email"], "new@test.com");
    assert_eq!(body["display_name"], "New Name");
}

#[tokio::test]
async fn update_user_set_admin_flag() {
    let app = AdminTestApp::spawn().await;

    // Create a non-admin user
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "promote",
            "email": "promote@test.com",
            "display_name": "Promote Me",
            "password": "secret"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.unwrap();
    let user_id = created["id"].as_str().unwrap();
    assert_eq!(created["is_admin"], false);

    // Promote to admin
    let res = app
        .client
        .put(app.url(&format!("/api/v1/admin/users/{user_id}")))
        .json(&serde_json::json!({ "is_admin": true }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["is_admin"], true);
}

#[tokio::test]
async fn update_nonexistent_user_returns_404() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .put(app.url("/api/v1/admin/users/00000000-0000-0000-0000-000000000099"))
        .json(&serde_json::json!({ "email": "x@test.com" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn reset_password_succeeds() {
    let app = AdminTestApp::spawn().await;

    // Create a user
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "resetme",
            "email": "resetme@test.com",
            "display_name": "Reset Me",
            "password": "oldpass"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.unwrap();
    let user_id = created["id"].as_str().unwrap();

    // Reset password
    let res = app
        .client
        .put(app.url(&format!("/api/v1/admin/users/{user_id}/password")))
        .json(&serde_json::json!({ "new_password": "newpass123" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);

    // Verify login works with new password
    let bare_client = reqwest::Client::new();
    let login_res = bare_client
        .post(app.url("/api/v1/auth/local/login"))
        .json(&serde_json::json!({
            "username": "resetme",
            "password": "newpass123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_res.status(), 200);

    // Old password should no longer work
    let old_login_res = bare_client
        .post(app.url("/api/v1/auth/local/login"))
        .json(&serde_json::json!({
            "username": "resetme",
            "password": "oldpass"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(old_login_res.status(), 401);
}

#[tokio::test]
async fn reset_password_empty_returns_400() {
    let app = AdminTestApp::spawn().await;

    // Create a user
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "emptyresetpw",
            "email": "emptyresetpw@test.com",
            "display_name": "Empty Reset",
            "password": "oldpass"
        }))
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = create_res.json().await.unwrap();
    let user_id = created["id"].as_str().unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/admin/users/{user_id}/password")))
        .json(&serde_json::json!({ "new_password": "" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn reset_password_nonexistent_user_returns_404() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .put(app.url("/api/v1/admin/users/00000000-0000-0000-0000-000000000099/password"))
        .json(&serde_json::json!({ "new_password": "newpass" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_user_succeeds() {
    let app = AdminTestApp::spawn().await;

    // Create a user to delete
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "deleteme",
            "email": "deleteme@test.com",
            "display_name": "Delete Me",
            "password": "secret"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.unwrap();
    let user_id = created["id"].as_str().unwrap();

    // Delete the user
    let res = app
        .client
        .delete(app.url(&format!("/api/v1/admin/users/{user_id}")))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);

    // Verify user no longer appears in list
    let list_res = app
        .client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();
    let users: serde_json::Value = list_res.json().await.unwrap();
    let usernames: Vec<&str> = users
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert!(!usernames.contains(&"deleteme"));
}

#[tokio::test]
async fn delete_self_returns_400() {
    let app = AdminTestApp::spawn().await;

    // Get admin user's ID from the user list
    let list_res = app
        .client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();
    let users: serde_json::Value = list_res.json().await.unwrap();
    let admin_user = users
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["username"].as_str().unwrap() == "admin")
        .unwrap();
    let admin_id = admin_user["id"].as_str().unwrap();

    // Try to delete self
    let res = app
        .client
        .delete(app.url(&format!("/api/v1/admin/users/{admin_id}")))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn delete_user_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    // Create a user to attempt to delete
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/users"))
        .json(&serde_json::json!({
            "username": "victim",
            "email": "victim@test.com",
            "display_name": "Victim",
            "password": "secret"
        }))
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = create_res.json().await.unwrap();
    let user_id = created["id"].as_str().unwrap();

    // Non-admin tries to delete
    let res = app
        .non_admin_client
        .delete(app.url(&format!("/api/v1/admin/users/{user_id}")))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}
