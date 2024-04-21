use chrono::{NaiveDateTime, Utc};
use std::time::SystemTime;

pub fn get_current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap()
}

pub fn get_current_timestamp_str() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub fn opt_or_empty_string<T: ToString>(opt: Option<T>) -> String {
    match opt {
        Some(s) => s.to_string(),
        None => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::time::{
        get_current_timestamp, get_current_timestamp_str, opt_or_empty_string,
    };

    #[test]
    fn test_get_current_timestamp() {
        let timestamp = get_current_timestamp();

        assert!(timestamp > 0);
    }

    #[test]
    fn test_get_current_timestamp_str() {
        let timestamp = get_current_timestamp_str();

        assert!(timestamp.and_utc().timestamp() > 0);
    }

    #[test]
    fn test_opt_or_empty_string() {
        let opt = Some("test");

        assert_eq!(opt_or_empty_string(opt), "test".to_string());
    }
    #[test]
    fn test_opt_or_empty_string_with_empty_string() {
        let opt = Some("");

        assert_eq!(opt_or_empty_string(opt), "");
    }
    #[test]
    fn test_opt_or_empty_string_with_none() {
        let opt: Option<String> = None;

        assert_eq!(opt_or_empty_string(opt), "");
    }
}
