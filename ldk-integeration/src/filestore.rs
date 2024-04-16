use std::collections::HashMap;

use lightning::util::persist::KVStore;
use bitcoin::secp256k1::SecretKey;
use teos_common::TowerId;
use serde::Serialize;
use serde::Deserialize;
use watchtower_plugin::TowerStatus;

/// Summarized data associated with a given tower.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TowerInfo {
    pub net_addr: String,
    pub available_slots: u32,
    pub subscription_start: u32,
    pub subscription_expiry: u32,
    pub status: TowerStatus,
}

pub type TowerList = HashMap<TowerId,TowerInfo>;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UserInfo(pub SecretKey);


#[derive(Debug)]
pub enum Error {
    EncodingIssue,
    DecodingIssue,
    FetchingIssue,
    MissingForeignKey,
    MissingField,
    NotFound,
    TowerNotFound,
}

const PRIAMRY_NAMESPACE: &str = "watchtower";
const SECONDARY_NAMESPACE: &str = "version";
const USER_KEY: &str = "userkey";
const TOWERLIST_KEY: &str = "towerlistkey";

#[derive(Serialize, Deserialize)]
pub(crate) struct Filestore<T: KVStore>(T);
	
impl<T:KVStore> Filestore<T> {

    pub(crate) fn new(store:T) -> Self {
        Filestore(store)
    }

    pub fn write_user_details(&self, user_info: UserInfo) -> Result<(), Error> {
        let data = serde_json::to_string(&user_info).map_err(|_| Error::EncodingIssue )?;
        let encoded_details = data.as_bytes();
        self.store.write(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, USER_KEY, encoded_details);
        Ok(())
    }

    pub fn get_user_details(&self) -> Result<UserInfo, Error> {
        let data = self.store.read(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, USER_KEY).map_err(|_| Error::FetchingIssue)?;
        let encoded_user_details = data.as_slice();
        let secret: UserInfo = serde_json::from_slice(encoded_user_details).map_err(|_| Error::DecodingIssue)?;
        return Ok(secret);
    }

    pub fn get_towerlist(&self) -> Result<TowerList, Error> {
        let data = self.store.read(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, TOWERLIST_KEY).map_err(|_| Error::FetchingIssue)?;
        let encoded_towerlist = data.as_slice();
        let towerlist: TowerList = serde_json::from_slice(encoded_towerlist).map_err(|_| Error::DecodingIssue)?;
        return Ok(towerlist) 
    }

    pub fn get_tower(&self, tower_id: TowerId) -> Result<TowerInfo, Error> {
        let towerlist = self.get_towerlist()?;
        towerlist.get(&tower_id).ok_or(Error::TowerNotFound)
    }

    pub fn write_tower(&self, tower_id: TowerId, tower_info: TowerInfo) -> Result<(), Error> {
        let mut towers = self.read_towers()?;
        towers.insert(tower_id, tower_info );
        self.write_towers(towers)?;
        return Ok(()) 
    }

    pub fn write_towers(&self, towers: TowerList) -> Result<(),Error> {
        let data = serde_json::to_string(&towers).map_err(|_| Error::EncodingIssue )?;
        let encoded_tower_details = data.as_bytes();
        let _  = self.store.write(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, TOWERLIST_KEY, encoded_tower_details).map_err(|_| Error::EncodingIssue)?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use teos_common::test_utils::get_random_int;

    use super::*;
    use crate::test_utils::{
        get_random_user_info, TestStore
    };

    #[test]
    fn test_write_user_details() {
        let filestore =  Filestore::<TestStore>::new(TestStore::new());
        let user_info = get_random_user_info();
        let randoM_numbers = get_random_int();
        filestore.write_user_details(user_info);
    }

}