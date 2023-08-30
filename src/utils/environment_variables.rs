use std::env;

pub fn is_env_var_present_and_true(env_var: &str) -> bool {
    match env::var(env_var) {
        Ok(val) => val == "true"||val == "1"||val == "yes",
        Err(_) => false,
    }
}