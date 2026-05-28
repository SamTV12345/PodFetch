use uuid::Uuid;

/// Generate a new time-ordered (v7) identifier for a freshly created row.
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_is_v7_and_unique() {
        let a = new_id();
        let b = new_id();
        assert_eq!(a.get_version_num(), 7, "must be a v7 UUID");
        assert_ne!(a, b, "two ids must differ");
        // v7 is time-ordered: a was created before b.
        assert!(a <= b, "v7 ids should be monotonically non-decreasing");
    }
}
