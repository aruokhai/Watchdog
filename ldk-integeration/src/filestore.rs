use std::collections::HashMap;
use std::io::ErrorKind;

use lightning::util::persist::KVStore;
use bitcoin::secp256k1::SecretKey;
use serde::Serializer;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserInfo(pub SecretKey);


#[derive(Debug, PartialEq, Eq)]
pub enum FilestoreError {
    EncodingIssue,
    DecodingIssue,
    MissingForeignKey,
    MissingField,
    NotFound,
    TowerNotFound,
    UserNotFound,
    TowerlistNotFound,
    IOError(ErrorKind)

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

    pub fn write_user_details(&self, user_info: UserInfo) -> Result<(), FilestoreError> {
        let data = user_info.0.secret_bytes();
        let encoded_details = data.as_ref();
        self.0.write(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, USER_KEY, encoded_details).map_err(|err| FilestoreError::IOError(err.kind()))
    }

    pub fn get_user_details(&self) -> Result<UserInfo, FilestoreError> {
        let data= self.0.read(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, USER_KEY).map_err(|err| FilestoreError::IOError(err.kind()))?;
        let encoded_user_details = data.as_slice();
        
        let secret =  SecretKey::from_slice(encoded_user_details).map_err(|_| FilestoreError::DecodingIssue)?;
        let user_info = UserInfo(secret);
        return Ok(user_info);
    }

    pub fn get_towerlist(&self) -> Result<TowerList, FilestoreError> {
        let data = self.0.read(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, TOWERLIST_KEY).map_err(|err| FilestoreError::IOError(err.kind()))?;
        let encoded_towerlist = data.as_slice();
        let towerlist: TowerList = serde_json::from_slice(encoded_towerlist).map_err(|_| FilestoreError::DecodingIssue)?;
        return Ok(towerlist) 
    }

    pub fn get_tower(&self, tower_id: TowerId) -> Result<TowerInfo, FilestoreError> {
        let towerlist = self.get_towerlist()?;
        let tower = towerlist.get(&tower_id).ok_or(FilestoreError::TowerNotFound)?;
        Ok(tower.to_owned())
    }

    pub fn write_tower(&self, tower_id: TowerId, tower_info: TowerInfo) -> Result<(), FilestoreError> {
        let mut towers =  self.get_towerlist().map_or_else(|err| {
            if err == FilestoreError::IOError(ErrorKind::NotFound) {
                return Ok(HashMap::new())
            };
            Err(err)
        },|towerlist| Ok(towerlist))?;
        towers.insert(tower_id, tower_info );
        self.write_towerlist(towers)
    }

    pub fn write_towerlist(&self, towers: TowerList) -> Result<(),FilestoreError> {
        let data = serde_json::to_string(&towers).map_err(|_| FilestoreError::EncodingIssue )?;
        let encoded_tower_details = data.as_bytes();
        self.0.write(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, TOWERLIST_KEY, encoded_tower_details).map_err(|err| FilestoreError::IOError(err.kind()))
    }

}




#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{
        get_random_tower, get_random_user_info, TestStore
    };

    #[test]
    fn test_write_user_details() {
        let filestore =  Filestore::<TestStore>::new(TestStore::new());
        let user_info = get_random_user_info();
        let _ = filestore.write_user_details(user_info);
        assert_eq!(filestore.0.store.into_inner().values().len(), 1)
    }

    #[test]
    fn test_get_user_details() {
        let filestore =  Filestore::<TestStore>::new(TestStore::new());
        let user_info = get_random_user_info();
        let _ = filestore.write_user_details(user_info.clone());
        let data = filestore.get_user_details().unwrap();
        assert_eq!(data, user_info)
    }

    #[test]
    fn test_write_tower() {
        let filestore =  Filestore::<TestStore>::new(TestStore::new());
        let towerinfo = get_random_tower();
        _ = filestore.write_tower(towerinfo.0, towerinfo.1);
        let store_length = filestore.0.store.into_inner().values().len();
        println!("len {}", store_length);
        assert_eq!(store_length, 1)
    }

    #[test]
    fn test_get_tower() {
        let filestore =  Filestore::<TestStore>::new(TestStore::new());
        let towerinfo = get_random_tower();
        _ = filestore.write_tower(towerinfo.0, towerinfo.1.clone());
        let data = filestore.get_tower(towerinfo.0).unwrap();
        assert_eq!(data, towerinfo.1);
        let data = filestore.get_tower(get_random_tower().0);
        assert_eq!(data, Err(FilestoreError::TowerNotFound));
    }

}