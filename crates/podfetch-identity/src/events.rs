//! Events published by the identity context.

use podfetch_events::Event;
use podfetch_kernel::UserId;
use serde::{Deserialize, Serialize};

/// A user was deleted. Contexts that keep data per user remove theirs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserDeleted {
    pub user_id: UserId,
    pub username: String,
}

impl Event for UserDeleted {
    const TYPE: &'static str = "identity.user_deleted";

    fn aggregate_id(&self) -> String {
        self.user_id.to_string()
    }
}
