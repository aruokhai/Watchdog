use std::collections::HashMap;
use teos_common::TowerId;
use teos_common::UserId;
use watchtower_plugin::TowerStatus;
use watchtower_plugin::TowerSummary;

use crate::filestore::TowerInfo;

pub fn set_tower_status(map: &mut HashMap<TowerId,TowerInfo> , tower_id: TowerId, status: TowerStatus) {
    if let Some(tower) = map.get_mut(&tower_id) {
        if tower.status != status {
            tower.status = status
        } else {
            log::debug!("{tower_id} status is already {status}")
        }
    } else {
        log::error!("Cannot change tower status to {status}. Unknown tower_id: {tower_id}");
    }
}