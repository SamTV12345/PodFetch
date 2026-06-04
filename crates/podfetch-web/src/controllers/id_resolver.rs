use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use uuid::Uuid;

/// A podcast/episode path segment that is either a UUID (new links) or a
/// legacy integer id (old links).
#[derive(Debug, PartialEq, Eq)]
pub enum ResolvedId {
    Uuid(Uuid),
    Legacy(i64),
}

/// Parse a `{id}` path segment: prefer UUID, fall back to legacy integer.
pub fn parse_resolved_id(segment: &str) -> Result<ResolvedId, CustomError> {
    if let Ok(uuid) = Uuid::parse_str(segment) {
        return Ok(ResolvedId::Uuid(uuid));
    }
    if let Ok(legacy) = segment.parse::<i64>() {
        return Ok(ResolvedId::Legacy(legacy));
    }
    Err(CustomErrorInner::BadRequest(format!("'{segment}' is not a valid id"), Warning).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_uuid() {
        let u = "0192f3a1-7c42-7e8b-8b2a-2b1c3d4e5f60";
        // `CustomError` is not `PartialEq`, so compare the unwrapped `Ok` value.
        assert_eq!(
            parse_resolved_id(u).unwrap(),
            ResolvedId::Uuid(Uuid::parse_str(u).unwrap())
        );
    }
    #[test]
    fn parses_legacy_integer() {
        assert_eq!(parse_resolved_id("42").unwrap(), ResolvedId::Legacy(42));
    }
    #[test]
    fn rejects_garbage() {
        assert!(parse_resolved_id("not-an-id").is_err());
    }
}
