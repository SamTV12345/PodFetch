#[cfg(test)]
pub mod tests {
    use crate::commands::startup::tests::TestServerWrapper;
    use base64::Engine;
    use base64::engine::general_purpose;
    use podfetch_domain::user::User;

    pub async fn create_auth_gpodder(server: &mut TestServerWrapper<'_>, user: &User) {
        let encoded_auth =
            general_purpose::STANDARD.encode(format!("{}:{}", user.username, "password"));
        server.test_server.clear_headers();
        server
            .test_server
            .add_header("Authorization", format!("Basic {encoded_auth}"));

        let response = {
            let server_ref = &mut server.test_server;
            server_ref
                .post(&format!("/api/2/auth/{}/login.json", user.username))
                .await
        };
        assert_eq!(response.status_code().as_u16(), 200);
        let cookie_binding = response.cookies();
        server
            .test_server
            .add_cookie(cookie_binding.get("sessionid").unwrap().clone());
        assert!(response.status_code().is_success());
        assert!(response.cookies().get("sessionid").is_some());
    }
}
