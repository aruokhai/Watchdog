use std::{collections::{HashMap, HashSet}, io, vec};

use bitcoin::secp256k1::SecretKey;
use lightning::util::persist::KVStore;
use crate::filestore::UserInfo;
use rand::Rng;




pub fn get_random_user_info() -> UserInfo {
    let mut rng = rand::thread_rng();

    let key = SecretKey::from_slice(&rng.gen::<[u8; 32]>()).unwrap();

    UserInfo(key)
}

pub struct TestStore {
    pub store: HashMap<String,HashMap<String, Vec<u8>>>,
    pub keys: HashMap<String, HashSet<String>>,

}

impl TestStore {
    pub fn new() -> Self {
        TestStore{ store: HashMap::new(), keys: HashMap::new() }
    }
}


impl KVStore for TestStore {

    fn read(&self, primary_namespace: &str, secondary_namespace: &str, key: &str) -> Result<Vec<u8>, io::Error> {
        let store_key = format!("{}{}{}",primary_namespace, secondary_namespace,key);
        self.store.get(&store_key).unwrap()
    }
	
	fn write(&self, primary_namespace: &str, secondary_namespace: &str, key: &str, buf: &[u8]) -> Result<(), io::Error>{
        let namespace = format!("{}{}",primary_namespace, secondary_namespace);
        let store_key = format!("{}{}",namespace,key);
        let namespace_keys = self.keys.get(&namespace).unwrap().to_owned();
        namespace_keys.insert(store_key);
        self.counter.insert(namespace, namespace_keys);
        let _ = self.store.insert(&store_key, buf.to_vec());
        Ok(())
    }

    fn remove(&self, primary_namespace: &str, secondary_namespace: &str, key: &str, lazy: bool) -> Result<(), io::Error> {
        let namespace = format!("{}{}",primary_namespace, secondary_namespace);
        let store_key = format!("{}{}",namespace,key);
        let namespace_keys = self.keys.get(&namespace).unwrap().to_owned();
        namespace_keys.insert(store_key);
        self.counter.remove(namespace, namespace_keys);
        self.store.remove(&store_key);
        Ok(())
    }
	
	fn list(&self, primary_namespace: &str, secondary_namespace: &str) -> Result<Vec<String>, io::Error> {
        let namespace = format!("{}{}",primary_namespace, secondary_namespace);
        self.keys.get(&namespace).unwrap().into()

    }
    
}
