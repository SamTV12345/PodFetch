#[cfg(test)]
pub mod tests {
    use crate::commands::startup::tests::TestServerWrapper;
    use crate::models::user::User;
    use base64::engine::general_purpose;
    use base64::Engine;

    pub fn create_basic_header(username: &str, password: &str) -> String {
        general_purpose::STANDARD.encode(format!("{}:{}", username, password))
    }

    pub async fn create_auth_gpodder(server: &mut TestServerWrapper<'_>, user: &User) {
        let encoded_auth =
            general_purpose::STANDARD.encode(format!("{}:{}", user.username, "password"));
        server.test_server.clear_headers();
        server
            .test_server
            .add_header("Authorization", format!("Basic {}", encoded_auth));

        let response = {
            let server_ref = &mut server.test_server;
            server_ref
                .post(&format!("/api/2/auth/{}/login.json", user.username))
                .await
        };
        assert_eq!(response.status_code().as_u16(), 200);
        // get devices
        let cookie_binding = response.cookies();
        server
            .test_server
            .add_cookie(cookie_binding.get("sessionid").unwrap().clone());
        assert!(response.status_code().is_success());
        assert!(response.cookies().get("sessionid").is_some());
    }
}
