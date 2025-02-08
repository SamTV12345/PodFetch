

pub mod tests {
    use axum_test::{TestServer};
    use base64::engine::general_purpose;
    use base64::Engine;
    use crate::models::user::User;

    pub fn create_basic_header(username: &str, password: &str) -> String {
        general_purpose::STANDARD.encode(format!("{}:{}", username, password))
    }

    pub async fn create_auth_gpodder(server: &mut TestServer, user: &User) {
        let encoded_auth =
            general_purpose::STANDARD.encode(format!("{}:{}", user.username, "password"));
        server.clear_headers();
        server.add_header("Authorization", format!("Basic {}", encoded_auth));

        let response = server
            .post(&format!("/api/2/auth/{}/login.json", user.username))
            .await;
        // get devices
        let cookie_binding = response.cookies();
        server.add_cookie(cookie_binding.get("sessionid").unwrap().clone());
        assert!(response.status_code().is_success());
        assert!(response.cookies().get("sessionid").is_some());
    }
}
