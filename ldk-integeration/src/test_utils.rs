use std::{cell::{Cell, RefCell}, collections::{HashMap, HashSet}, io, vec};

use bitcoin::secp256k1::SecretKey;
use lightning::util::persist::KVStore;
use teos_common::{test_utils::get_random_user_id, UserId};
use watchtower_plugin::TowerStatus;
use crate::filestore::UserInfo;
use crate::filestore::TowerInfo;
use rand::Rng;
use teos_common::TowerId;



pub fn get_random_user_info() -> UserInfo {
    let mut rng = rand::thread_rng();
    let key = SecretKey::from_slice(&rng.gen::<[u8; 32]>()).unwrap();
    UserInfo(key)
}

pub fn get_random_tower() -> (TowerId, TowerInfo) {
    let tower_id: TowerId = get_random_user_id();
    let mut rng = rand::thread_rng();
    let tower_info = TowerInfo { net_addr: "dummyaddress".to_string(),  available_slots: rng.gen::<u32>(), subscription_start: rng.gen::<u32>(), subscription_expiry: rng.gen::<u32>(), status: TowerStatus::Reachable};
    (tower_id, tower_info)
}



pub struct TestStore {
    pub store: RefCell<HashMap<String,Vec<u8>>>,
    pub keys: RefCell<HashMap<String, HashSet<String>>>,

}

impl TestStore {
    pub fn new() -> Self {
        TestStore{ store: RefCell::new(HashMap::new()), keys: RefCell::new(HashMap::new()) }
    }
}


impl KVStore for TestStore {

    fn read(&self, primary_namespace: &str, secondary_namespace: &str, key: &str) -> Result<Vec<u8>, io::Error> {
        let store_key = format!("{}{}{}",primary_namespace, secondary_namespace,key);
        let inner_store = self.store.clone().into_inner();
        let data = inner_store.get(&store_key).ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))?;
        Ok(data.to_owned())
    }
	
	fn write(&self, primary_namespace: &str, secondary_namespace: &str, key: &str, buf: &[u8]) -> Result<(), io::Error>{
        let namespace = format!("{}{}",primary_namespace, secondary_namespace);
        let store_key = format!("{}{}",namespace,key);
        self.keys.replace_with(|map|{ 
            let mut namespace_keys = map.get(&namespace).or(Some(&HashSet::new())).unwrap().to_owned();
            namespace_keys.insert(store_key.clone());
            map.insert(namespace, namespace_keys);
            map.to_owned()
        });
        self.store.replace_with(|map|{ 
            map.insert(store_key, buf.to_vec());
            map.to_owned()
        });
        Ok(())
    }

    fn remove(&self, primary_namespace: &str, secondary_namespace: &str, key: &str, lazy: bool) -> Result<(), io::Error> {
        let namespace = format!("{}{}",primary_namespace, secondary_namespace);
        let store_key = format!("{}{}",namespace,key);
        self.keys.replace_with(|map|{ 
            let mut namespace_keys = map.get(&namespace).unwrap().to_owned();
            namespace_keys.remove(&store_key);
            map.insert(namespace, namespace_keys);
            map.to_owned()
        });
        self.store.replace_with(|map|{ 
            map.remove(&store_key);
            map.to_owned()
        });
        Ok(())
    }
	
	fn list(&self, primary_namespace: &str, secondary_namespace: &str) -> Result<Vec<String>, io::Error> {
        let namespace = format!("{}{}",primary_namespace, secondary_namespace);
        Ok(self.keys.clone().into_inner().get(&namespace).unwrap().to_owned().into_iter().collect::<Vec<String>>())

    }
    
}
