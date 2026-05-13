//! Test helpers for the audiobookshelf-compatible API.

#[cfg(test)]
pub mod tests {
    use crate::test_support::tests::TestServerWrapper;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use podfetch_domain::user::User;
    use serde_json::Value;
    use serde_json::json;

    /// Creates a user with a known password ("password"), POSTs /login to mint a
    /// Bearer token (the user's `users.api_key`), and adds the
    /// `Authorization: Bearer <token>` header to subsequent requests.
    pub async fn login_audiobookshelf(server: &mut TestServerWrapper<'_>, user: &User) -> String {
        server.test_server.clear_headers();
        let response = server
            .test_server
            .post("/login")
            .json(&json!({ "username": user.username, "password": "password" }))
            .await;
        assert_eq!(
            response.status_code().as_u16(),
            200,
            "POST /login failed: {:?}",
            response.text()
        );
        let body: Value = response.json();
        let token = body["user"]["token"]
            .as_str()
            .expect("login response missing user.token")
            .to_string();
        server
            .test_server
            .add_header("Authorization", format!("Bearer {token}"));
        token
    }

    pub fn random_user_for_audiobookshelf() -> User {
        UserTestDataBuilder::new().build()
    }
}
