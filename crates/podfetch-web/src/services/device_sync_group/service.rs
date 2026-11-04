use common_infrastructure::error::CustomError;
use podfetch_domain::device_sync_group::{DeviceSyncGroup, DeviceSyncGroupRepository};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct DeviceSyncGroupService {
    repository: Arc<dyn DeviceSyncGroupRepository<Error = CustomError>>,
}

impl DeviceSyncGroupService {
    pub fn new(repository: Arc<dyn DeviceSyncGroupRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<DeviceSyncGroup>, CustomError> {
        self.repository.get_by_user_id(user_id)
    }

    /// Validate a complete sync group configuration (one list of device ids
    /// per group, as in GPodder's `synchronized` list) and turn it into rows.
    ///
    /// Every device must be in `known_devices` and may appear in at most one
    /// group. Groups with fewer than two devices synchronize nothing and are
    /// dropped. Group ids are renumbered from 1.
    pub fn build_groups(
        user_id: Uuid,
        groups: &[Vec<String>],
        known_devices: &std::collections::HashSet<String>,
    ) -> Result<Vec<DeviceSyncGroup>, String> {
        let mut seen = std::collections::HashSet::new();
        let mut rows = vec![];
        let mut group_id = 0;
        for group in groups {
            let mut members: Vec<&String> = vec![];
            for device_id in group {
                if !known_devices.contains(device_id) {
                    return Err(format!("Unknown device '{device_id}'"));
                }
                if !seen.insert(device_id.clone()) {
                    return Err(format!("Device '{device_id}' is in more than one group"));
                }
                members.push(device_id);
            }
            if members.len() < 2 {
                continue;
            }
            group_id += 1;
            rows.extend(members.into_iter().map(|device_id| DeviceSyncGroup {
                id: podfetch_domain::ids::new_id(),
                user_id,
                group_id,
                device_id: device_id.clone(),
            }));
        }
        Ok(rows)
    }

    /// The group id `device_id` belongs to, if any.
    pub fn group_of(groups: &[DeviceSyncGroup], device_id: &str) -> Option<i32> {
        groups
            .iter()
            .find(|g| g.device_id == device_id)
            .map(|g| g.group_id)
    }

    pub fn replace_all(
        &self,
        user_id: Uuid,
        groups: Vec<DeviceSyncGroup>,
    ) -> Result<(), CustomError> {
        self.repository.replace_all(user_id, groups)
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceSyncGroupService;
    use std::collections::HashSet;
    use uuid::Uuid;

    fn known(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    fn groups(raw: &[&[&str]]) -> Vec<Vec<String>> {
        raw.iter()
            .map(|g| g.iter().map(|s| s.to_string()).collect())
            .collect()
    }

    #[test]
    fn builds_and_renumbers_groups() {
        let rows = DeviceSyncGroupService::build_groups(
            Uuid::new_v4(),
            &groups(&[&["phone", "webview"], &["tablet"], &["laptop", "desktop"]]),
            &known(&["phone", "webview", "tablet", "laptop", "desktop"]),
        )
        .unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(DeviceSyncGroupService::group_of(&rows, "phone"), Some(1));
        assert_eq!(DeviceSyncGroupService::group_of(&rows, "laptop"), Some(2));
        // A single device synchronizes nothing.
        assert_eq!(DeviceSyncGroupService::group_of(&rows, "tablet"), None);
    }

    #[test]
    fn rejects_unknown_devices() {
        let result = DeviceSyncGroupService::build_groups(
            Uuid::new_v4(),
            &groups(&[&["phone", "someone-elses-device"]]),
            &known(&["phone"]),
        );
        assert!(result.is_err());
    }

    #[test]
    fn rejects_device_in_two_groups() {
        let result = DeviceSyncGroupService::build_groups(
            Uuid::new_v4(),
            &groups(&[&["phone", "webview"], &["phone", "tablet"]]),
            &known(&["phone", "webview", "tablet"]),
        );
        assert!(result.is_err());
    }
}
