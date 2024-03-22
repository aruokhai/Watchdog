use std::collections::HashMap;

use lightning::util::persist::{self, KVStore};
use bitcoin::secp256k1::SecretKey;
use serde_json::json;
use teos_common::receipts::RegistrationReceipt;
use teos_common::{TowerId, UserId};
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


#[derive(Debug)]
pub enum Error {
    EncodingIssue,
    DecodingIssue,
    FetchingIssue,
    MissingForeignKey,
    MissingField,
    NotFound,
}

const PRIAMRY_NAMESPACE: &str = "watchtower";
const SECONDARY_NAMESPACE: &str = "version";

#[derive(Serialize, Deserialize)]


pub(crate) struct Filestore<T: KVStore> {
	store: T,
}




impl<T:KVStore> Filestore<T> {

    pub(crate) fn new(store:T) -> Self {
        Filestore{store}
    }

    pub fn write_user_details(&self, user_sk: SecretKey) -> Result<(), Error> {
        let data = serde_json::to_string(&user_sk).map_err(|_| Error::EncodingIssue )?;
        let encoded_details = data.as_bytes();
        self.store.write(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, "user_details", encoded_details);
        Ok(())
    }

    pub fn read_user_details(&self) -> Result<SecretKey, Error> {
        let data = self.store.read(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, "user_details").map_err(|_| Error::FetchingIssue)?;
        let encoded_user_details = data.as_slice();
        let secret: SecretKey = serde_json::from_slice(encoded_user_details).map_err(|_| Error::DecodingIssue)?;
        return Ok(secret);
    }

    pub fn read_towers(&self) -> Result<HashMap<TowerId,TowerInfo>, Error> {
        let data = self.store.read(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, "tower_details").map_err(|_| Error::FetchingIssue)?;
        let encoded_tower_details = data.as_slice();
        let towers: HashMap<TowerId,TowerInfo> = serde_json::from_slice(encoded_tower_details).map_err(|_| Error::DecodingIssue)?;
        return Ok(towers) 
    }

    pub fn write_tower(&self, tower_id: TowerId, tower_adrress: String, tower_details: RegistrationReceipt) -> Result<(), Error> {
        let mut towers = self.read_towers()?;
        let new_tower = TowerInfo{ net_addr: tower_adrress, available_slots: tower_details.available_slots(), subscription_start: tower_details.subscription_start(), subscription_expiry: tower_details.subscription_expiry(), status: TowerStatus::Reachable };
        towers.insert(tower_id, new_tower );
        self.write_towers(towers)?;
        return Ok(()) 
    }

    pub fn write_towers(&self, towers: HashMap<TowerId,TowerInfo>) -> Result<(),Error> {
        let data = serde_json::to_string(&towers).map_err(|_| Error::EncodingIssue )?;
        let encoded_tower_details = data.as_bytes();
        let _  = self.store.write(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, "tower_details", encoded_tower_details).map_err(|_| Error::EncodingIssue)?;
        Ok(())
    }

}

