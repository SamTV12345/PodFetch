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

    /// Whether `device_id` shares its subscriptions with `peer_device_id`.
    pub fn is_synced_with(
        groups: &[DeviceSyncGroup],
        device_id: &str,
        peer_device_id: &str,
    ) -> bool {
        let group_of = |id: &str| {
            groups
                .iter()
                .find(|g| g.device_id == id)
                .map(|g| g.group_id)
        };
        matches!((group_of(device_id), group_of(peer_device_id)), (Some(a), Some(b)) if a == b)
    }

    /// Put `device_id` into the same sync group as `peer_device_id`
    /// (`enabled = true`) or take it out of that group (`enabled = false`).
    ///
    /// Enabling merges the device's current group into the peer's group, so
    /// devices that were already synchronized with it stay synchronized.
    /// Groups left with a single member are dropped.
    pub fn set_synced_with(
        &self,
        user_id: Uuid,
        device_id: &str,
        peer_device_id: &str,
        enabled: bool,
    ) -> Result<(), CustomError> {
        let groups = self.repository.get_by_user_id(user_id)?;
        let groups = Self::apply_sync_change(groups, user_id, device_id, peer_device_id, enabled);
        self.repository.replace_all(user_id, groups)
    }

    fn apply_sync_change(
        mut groups: Vec<DeviceSyncGroup>,
        user_id: Uuid,
        device_id: &str,
        peer_device_id: &str,
        enabled: bool,
    ) -> Vec<DeviceSyncGroup> {
        let group_of = |groups: &[DeviceSyncGroup], id: &str| {
            groups
                .iter()
                .find(|g| g.device_id == id)
                .map(|g| g.group_id)
        };
        let peer_group = group_of(&groups, peer_device_id);

        if enabled {
            let target = peer_group
                .unwrap_or_else(|| groups.iter().map(|g| g.group_id).max().unwrap_or(0) + 1);
            if peer_group.is_none() {
                groups.push(DeviceSyncGroup {
                    id: podfetch_domain::ids::new_id(),
                    user_id,
                    group_id: target,
                    device_id: peer_device_id.to_string(),
                });
            }
            match group_of(&groups, device_id) {
                Some(current) if current == target => {}
                Some(current) => groups
                    .iter_mut()
                    .filter(|g| g.group_id == current)
                    .for_each(|g| g.group_id = target),
                None => groups.push(DeviceSyncGroup {
                    id: podfetch_domain::ids::new_id(),
                    user_id,
                    group_id: target,
                    device_id: device_id.to_string(),
                }),
            }
        } else if let Some(peer_group) = peer_group {
            groups.retain(|g| !(g.device_id == device_id && g.group_id == peer_group));
        }

        // A group with a single device synchronizes nothing.
        let mut sizes = std::collections::HashMap::new();
        for g in &groups {
            *sizes.entry(g.group_id).or_insert(0) += 1;
        }
        groups.retain(|g| sizes[&g.group_id] > 1);
        groups
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
    use podfetch_domain::device_sync_group::DeviceSyncGroup;
    use uuid::Uuid;

    fn group(user_id: Uuid, group_id: i32, device_id: &str) -> DeviceSyncGroup {
        DeviceSyncGroup {
            id: Uuid::new_v4(),
            user_id,
            group_id,
            device_id: device_id.to_string(),
        }
    }

    fn members(groups: &[DeviceSyncGroup], peer: &str) -> Vec<String> {
        let mut ids: Vec<String> = groups
            .iter()
            .filter(|g| DeviceSyncGroupService::is_synced_with(groups, &g.device_id, peer))
            .map(|g| g.device_id.clone())
            .collect();
        ids.sort();
        ids
    }

    #[test]
    fn enabling_creates_group_with_peer() {
        let user = Uuid::new_v4();
        let groups =
            DeviceSyncGroupService::apply_sync_change(vec![], user, "phone", "webview", true);
        assert_eq!(members(&groups, "webview"), vec!["phone", "webview"]);
    }

    #[test]
    fn enabling_merges_existing_group_into_peer_group() {
        let user = Uuid::new_v4();
        let groups = vec![
            group(user, 1, "phone"),
            group(user, 1, "tablet"),
            group(user, 2, "webview"),
            group(user, 2, "desktop"),
        ];
        let groups =
            DeviceSyncGroupService::apply_sync_change(groups, user, "phone", "webview", true);
        assert_eq!(
            members(&groups, "webview"),
            vec!["desktop", "phone", "tablet", "webview"]
        );
    }

    #[test]
    fn disabling_removes_device_and_drops_single_member_groups() {
        let user = Uuid::new_v4();
        let groups = vec![group(user, 1, "phone"), group(user, 1, "webview")];
        let groups =
            DeviceSyncGroupService::apply_sync_change(groups, user, "phone", "webview", false);
        assert!(groups.is_empty());
    }

    #[test]
    fn disabling_keeps_other_members() {
        let user = Uuid::new_v4();
        let groups = vec![
            group(user, 1, "phone"),
            group(user, 1, "tablet"),
            group(user, 1, "webview"),
        ];
        let groups =
            DeviceSyncGroupService::apply_sync_change(groups, user, "phone", "webview", false);
        assert_eq!(members(&groups, "webview"), vec!["tablet", "webview"]);
        assert!(!DeviceSyncGroupService::is_synced_with(
            &groups, "phone", "webview"
        ));
    }
}
