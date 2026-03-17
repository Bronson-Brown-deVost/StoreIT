use crate::common::AdminTestApp;

#[tokio::test]
async fn list_groups_returns_default_group() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/admin/groups"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let groups = body.as_array().unwrap();

    // AdminTestApp seeds a "default" group
    assert!(!groups.is_empty());
    let names: Vec<&str> = groups.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"default"));
}

#[tokio::test]
async fn list_groups_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .non_admin_client
        .get(app.url("/api/v1/admin/groups"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn list_groups_unauthenticated_returns_401() {
    let app = AdminTestApp::spawn().await;

    let bare = reqwest::Client::new();
    let res = bare
        .get(app.url("/api/v1/admin/groups"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn create_group_succeeds() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "test-group" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "test-group");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
}

#[tokio::test]
async fn create_group_empty_name_returns_400() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn create_group_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .non_admin_client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "sneaky-group" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn delete_group_succeeds() {
    let app = AdminTestApp::spawn().await;

    // Create a group to delete (no members)
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "to-delete" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.unwrap();
    let group_id = created["id"].as_str().unwrap();

    // Delete the group
    let res = app
        .client
        .delete(app.url(&format!("/api/v1/admin/groups/{group_id}")))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);

    // Verify the group no longer appears in the list
    let list_res = app
        .client
        .get(app.url("/api/v1/admin/groups"))
        .send()
        .await
        .unwrap();
    let groups: serde_json::Value = list_res.json().await.unwrap();
    let names: Vec<&str> = groups
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"to-delete"));
}

#[tokio::test]
async fn delete_group_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    // Create a group via admin
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "protected" }))
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = create_res.json().await.unwrap();
    let group_id = created["id"].as_str().unwrap();

    // Non-admin tries to delete
    let res = app
        .non_admin_client
        .delete(app.url(&format!("/api/v1/admin/groups/{group_id}")))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn list_group_members_returns_members() {
    let app = AdminTestApp::spawn().await;

    // Get the default group ID
    let list_res = app
        .client
        .get(app.url("/api/v1/admin/groups"))
        .send()
        .await
        .unwrap();
    let groups: serde_json::Value = list_res.json().await.unwrap();
    let default_group = groups
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["name"].as_str().unwrap() == "default")
        .unwrap();
    let group_id = default_group["id"].as_str().unwrap();

    // List members of the default group
    let res = app
        .client
        .get(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let members = body.as_array().unwrap();

    // AdminTestApp adds both admin and non-admin to the default group
    assert!(members.len() >= 2);
    for member in members {
        assert!(member["user"]["id"].is_string());
        assert!(member["user"]["username"].is_string());
        assert!(member["role"].is_string());
    }
}

#[tokio::test]
async fn list_group_members_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    let list_res = app
        .client
        .get(app.url("/api/v1/admin/groups"))
        .send()
        .await
        .unwrap();
    let groups: serde_json::Value = list_res.json().await.unwrap();
    let group_id = groups.as_array().unwrap()[0]["id"].as_str().unwrap();

    let res = app
        .non_admin_client
        .get(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn add_group_member_succeeds() {
    let app = AdminTestApp::spawn().await;

    // Create a new group
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "member-test" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let group: serde_json::Value = create_res.json().await.unwrap();
    let group_id = group["id"].as_str().unwrap();

    // Get a user ID to add
    let users_res = app
        .client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();
    let users: serde_json::Value = users_res.json().await.unwrap();
    let user_id = users.as_array().unwrap()[0]["id"].as_str().unwrap();

    // Add the user to the group
    let res = app
        .client
        .post(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .json(&serde_json::json!({
            "user_id": user_id,
            "role": "member"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);

    // Verify the member appears in the group
    let members_res = app
        .client
        .get(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .send()
        .await
        .unwrap();
    let members: serde_json::Value = members_res.json().await.unwrap();
    let member_ids: Vec<&str> = members
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["user"]["id"].as_str().unwrap())
        .collect();
    assert!(member_ids.contains(&user_id));
}

#[tokio::test]
async fn add_group_member_with_owner_role() {
    let app = AdminTestApp::spawn().await;

    // Create a new group
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "owner-test" }))
        .send()
        .await
        .unwrap();
    let group: serde_json::Value = create_res.json().await.unwrap();
    let group_id = group["id"].as_str().unwrap();

    // Get a user ID
    let users_res = app
        .client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();
    let users: serde_json::Value = users_res.json().await.unwrap();
    let user_id = users.as_array().unwrap()[0]["id"].as_str().unwrap();

    // Add as owner
    let res = app
        .client
        .post(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .json(&serde_json::json!({
            "user_id": user_id,
            "role": "owner"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn add_group_member_invalid_role_returns_400() {
    let app = AdminTestApp::spawn().await;

    // Create a new group
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "bad-role-test" }))
        .send()
        .await
        .unwrap();
    let group: serde_json::Value = create_res.json().await.unwrap();
    let group_id = group["id"].as_str().unwrap();

    // Get a user ID
    let users_res = app
        .client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();
    let users: serde_json::Value = users_res.json().await.unwrap();
    let user_id = users.as_array().unwrap()[0]["id"].as_str().unwrap();

    let res = app
        .client
        .post(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .json(&serde_json::json!({
            "user_id": user_id,
            "role": "superadmin"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn add_group_member_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    let list_res = app
        .client
        .get(app.url("/api/v1/admin/groups"))
        .send()
        .await
        .unwrap();
    let groups: serde_json::Value = list_res.json().await.unwrap();
    let group_id = groups.as_array().unwrap()[0]["id"].as_str().unwrap();

    let users_res = app
        .client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();
    let users: serde_json::Value = users_res.json().await.unwrap();
    let user_id = users.as_array().unwrap()[0]["id"].as_str().unwrap();

    let res = app
        .non_admin_client
        .post(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .json(&serde_json::json!({
            "user_id": user_id,
            "role": "member"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn remove_group_member_succeeds() {
    let app = AdminTestApp::spawn().await;

    // Create a new group and add a member
    let create_res = app
        .client
        .post(app.url("/api/v1/admin/groups"))
        .json(&serde_json::json!({ "name": "remove-test" }))
        .send()
        .await
        .unwrap();
    let group: serde_json::Value = create_res.json().await.unwrap();
    let group_id = group["id"].as_str().unwrap();

    // Get a user ID
    let users_res = app
        .client
        .get(app.url("/api/v1/admin/users"))
        .send()
        .await
        .unwrap();
    let users: serde_json::Value = users_res.json().await.unwrap();
    let user_id = users.as_array().unwrap()[0]["id"].as_str().unwrap();

    // Add the member
    let add_res = app
        .client
        .post(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .json(&serde_json::json!({
            "user_id": user_id,
            "role": "member"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(add_res.status(), 200);

    // Remove the member
    let res = app
        .client
        .delete(app.url(&format!(
            "/api/v1/admin/groups/{group_id}/members/{user_id}"
        )))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);

    // Verify member is no longer in the group
    let members_res = app
        .client
        .get(app.url(&format!("/api/v1/admin/groups/{group_id}/members")))
        .send()
        .await
        .unwrap();
    let members: serde_json::Value = members_res.json().await.unwrap();
    let member_ids: Vec<&str> = members
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["user"]["id"].as_str().unwrap())
        .collect();
    assert!(!member_ids.contains(&user_id));
}

#[tokio::test]
async fn remove_group_member_non_admin_returns_403() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .non_admin_client
        .delete(app.url(
            "/api/v1/admin/groups/00000000-0000-0000-0000-000000000001/members/00000000-0000-0000-0000-000000000002"
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}
