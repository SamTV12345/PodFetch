use serde_json::Value;

pub fn unwrap_string(value: &Value) -> String {
    return value.to_string().replace("\"", "");
}
