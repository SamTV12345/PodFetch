//! The shared kernel of PodFetch's bounded contexts.
//!
//! Deliberately tiny: only identifiers that several contexts use to refer to
//! each other's aggregates. Contexts never share entities, only these ids.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies a user, owned by the identity context.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(transparent)]
#[schema(value_type = String, format = "uuid")]
pub struct UserId(Uuid);

impl UserId {
    pub const fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<UserId> for Uuid {
    fn from(id: UserId) -> Self {
        id.0
    }
}

impl PartialEq<Uuid> for UserId {
    fn eq(&self, other: &Uuid) -> bool {
        self.0 == *other
    }
}

impl PartialEq<UserId> for Uuid {
    fn eq(&self, other: &UserId) -> bool {
        *self == other.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_id_round_trips_as_a_plain_uuid() {
        let uuid = Uuid::now_v7();
        let id = UserId::from(uuid);
        assert_eq!(id, uuid);
        assert_eq!(Uuid::from(id), uuid);
        assert_eq!(id.to_string(), uuid.to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{uuid}\""));
    }
}
