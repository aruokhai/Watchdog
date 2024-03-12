use std::collections::HashMap;

use lightning::util::persist::{self, KVStore};
use bitcoin::secp256k1::SecretKey;
use serde_json::json;
use teos_common::{TowerId, UserId};
use serde::Serialize;
use serde::Deserialize;
use watchtower_plugin::TowerSummary;


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
        let encoded_details = serde_json::to_string(&user_sk).map_err(|_| Error::EncodingIssue )?.as_bytes();
        self.store.write(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, "user_details", encoded_details);
        Ok(())
    }

    pub fn read_user_details(&self) -> Result<SecretKey, Error> {
        let encoded_user_details = self.store.read(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, "user_details").map_err(|_| Error::FetchingIssue)?.as_slice();
        let secret: SecretKey = serde_json::from_slice(encoded_user_details).map_err(|_| Error::DecodingIssue)?;
        return Ok(secret);
    }

    pub fn read_tower_details(&self) -> Result<HashMap<TowerId,TowerSummary>, Error> {
        let encoded_tower_details = self.store.read(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, "tower_details").map_err(|_| Error::FetchingIssue)?.as_slice();
        let towers: HashMap<TowerId,TowerSummary> = serde_json::from_slice(encoded_tower_details).map_err(|_| Error::DecodingIssue)?;
        return Ok(towers) 
    }

    pub fn write_tower_details(&self, towers: HashMap<TowerId,TowerSummary>) -> Result<(),Error> {
        let encoded_tower_details = serde_json::to_string(&towers).map_err(|_| Error::EncodingIssue )?.as_bytes();
        self.store.write(PRIAMRY_NAMESPACE, SECONDARY_NAMESPACE, "tower_details", encoded_tower_details);
        Ok(())
    }




}

