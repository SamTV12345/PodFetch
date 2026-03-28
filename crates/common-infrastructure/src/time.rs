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
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{get_current_timestamp, get_current_timestamp_str, opt_or_empty_string};

    #[test]
    fn test_get_current_timestamp() {
        assert!(get_current_timestamp() > 0);
    }

    #[test]
    fn test_get_current_timestamp_str() {
        assert!(get_current_timestamp_str().and_utc().timestamp() > 0);
    }

    #[test]
    fn test_opt_or_empty_string() {
        assert_eq!(opt_or_empty_string(Some("test")), "test");
        assert_eq!(opt_or_empty_string(Some("")), "");
        assert_eq!(opt_or_empty_string(None::<String>), "");
    }
}
