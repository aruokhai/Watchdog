use lightning::util::persist::{self, KVStore};
use teos_common::{TowerId, UserId};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::collections::HashMap;
use watchtower_plugin::{TowerSummary};
use persister::WatchtowerPersister;
use filestore::{Filestore};
use teos_common::cryptography;


pub mod persister; 
pub mod filestore;

pub(crate) struct WatchtowerMonitor<T: KVStore> {
	persister: WatchtowerPersister,
    tower_details: HashMap<TowerId,TowerSummary>,
    pub user_sk: SecretKey,
    /// The user identifier.
    pub user_id: UserId,
    pub storage: Filestore<T>,
}

impl<T:KVStore>  WatchtowerMonitor<T> {

    pub(crate) fn new(keystore: T) -> Self {
        let storage = Filestore::new(keystore);
        let (user_sk, user_id) = if let Ok(sk) = storage.read_user_details() {
            (
                sk,
                UserId(PublicKey::from_secret_key(&Secp256k1::new(), &sk)),
            )
        } else {
            log::info!("Watchtower client keys not found. Creating a fresh set");
            let (sk, pk) = cryptography::get_random_keypair();
            storage.write_user_details(sk);
            (sk, UserId(pk))
        };
        let towers = if let Ok(towers) = storage.read_tower_details() {
            towers
        } else {
            HashMap::new()
        };
        Self { persister: WatchtowerPersister{}, user_sk, user_id, storage, tower_details: towers}
    }

    pub fn register_client(&self, host: &str) ->  {



    

    
}

