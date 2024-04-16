use lightning::chain::chainmonitor::MonitorUpdateId;
use lightning::chain::{self, chainmonitor, channelmonitor};
use lightning::chain::transaction::OutPoint;
use lightning::sign;
use lightning::util::persist::{self, KVStore};
use net::http;
use teos_common::appointment::{Appointment, Locator};
use teos_common::receipts::RegistrationReceipt;
use teos_common::{net::NetAddr, TowerId, UserId};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::collections::HashMap;
use std::str::FromStr;
use watchtower_plugin::{TowerStatus, TowerSummary};
use watchtower_plugin::net::http::{self, RequestError};
use persister::WatchtowerPersister;
use filestore::{Filestore, TowerInfo};
use teos_common::cryptography;
use utils::set_tower_status;
use lightning_bitcoin::{Transaction, hash_types::Txid};

pub mod persister; 
pub mod filestore;
pub mod utils;
pub mod net;


pub(crate) struct WatchtowerMonitor<T: KVStore> {
	persister: WatchtowerPersister,
    towers: HashMap<TowerId,TowerInfo>,
    pub user_sk: SecretKey,
    /// The user identifier.
    pub user_id: UserId,
    pub storage: Filestore<T>,
}

pub enum Error {
    RequestError,
    InvalidReceipt(String),
    SubscriptionSlotError,
    SubscriptionExpiryError,
    StorageError,

}

pub struct RevokeableOutputData {

	pub commitment_txid: Txid,
	pub justice_transaction: Transaction,
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
            let _ = storage.write_user_details(sk);
            (sk, UserId(pk))
        };
        let towers = if let Ok(towers) = storage.read_towers() {
            towers
        } else {
            HashMap::new()
        };
        Self { persister: WatchtowerPersister{}, user_sk, user_id, storage, towers}
    }


    pub  fn add_update_appointment(& self, justice_data: RevokeableOutputData) -> Result<(), Error> {
        // TODO
        for (id, tower) in self.towers.clone() {

            // TODO: For now, to_self_delay is hardcoded to 42. Revisit and define it better / remove it when / if needed
            let locator = Locator::new(justice_data.commitment_txid);
            let appointment = Appointment::new(
                locator,
                cryptography::encrypt(
                    &justice_data.justice_transaction,
                    &justice_data.commitment_txid,
                )
                .unwrap(),
                42,
            );
            let signature = cryptography::sign(
                &appointment.to_vec(),
                &self.user_sk,
            )
            .unwrap();
            let result = http::add_update_appointment(id, tower_net_addr, &appointment, &signature);
    
            self.storage.write_tower(id, tower_adrress, tower_details).unwrap();
    
            if let Some(summary) = self.towers.get_mut(&tower_id) {
                summary.udpate(
                    tower_net_addr.to_owned(),
                    receipt.available_slots(),
                    receipt.subscription_start(),
                    receipt.subscription_expiry(),
                );
            } else {
                self.towers.insert(
                    tower_id,
                    TowerSummary::new(
                        tower_net_addr.to_owned(),
                        receipt.available_slots(),
                        receipt.subscription_start(),
                        receipt.subscription_expiry(),
                    ),
                );
        }
    }
        Ok(())
    }

    // pub async fn register_client(& mut self, host: String, tower_id : TowerId, user_id: UserId ) -> Result<(), Error> {
    //     let tower_net_addr = {
    //         let mut mod_host = host.clone();
    //         if !host.starts_with("http://") {
    //             mod_host = format!("http://{host}")
    //         }
    //         NetAddr::new(mod_host)
    //     };

    //     let receipt = http::register(tower_id, user_id, &tower_net_addr, &None)
    //         .await
    //         .map_err(|e| {
    //             let towers = &mut self.towers;
    //             if e.is_connection() && towers.contains_key(&tower_id) {
    //                 set_tower_status(towers, tower_id, TowerStatus::TemporaryUnreachable);
    //             }
    //             Error::RequestError
    //         })?;
    
    //     if !receipt.verify(&tower_id) {
    //         return Err(Error::InvalidReceipt(String::from_str("Registration receipt contains bad signature. Are you using the right tower_id?").unwrap()));
    //     }

    //     if let Some(tower) = self.towers.get(&tower_id) {
        
    //         if receipt.subscription_expiry() <= tower.subscription_expiry {
    //             return Err(Error::SubscriptionExpiryError);
    //         } else {
    //             if receipt.available_slots() <= tower.available_slots {
    //                 return Err(Error::SubscriptionSlotError);
    //             }
    //         }
    //     }

    //     self.storage
    //         .write_tower(tower_id, host, receipt)
    //         .unwrap();

    //     self.towers = self.storage.read_towers().map_err(|_| Error::StorageError )?;


    //     Ok(())

    // }


    

    
}

impl<Signer: lightning::sign::ecdsa::WriteableEcdsaChannelSigner, T:KVStore> chainmonitor::Persist<Signer>
	for WatchtowerMonitor<T>
{

    fn persist_new_channel(
		&self, funding_txo: OutPoint, data: &channelmonitor::ChannelMonitor<Signer>,
		id: MonitorUpdateId,
	) -> chain::ChannelMonitorUpdateStatus {
      
        chain::ChannelMonitorUpdateStatus::Completed

    }

	

    fn update_persisted_channel(
		&self, funding_txo: OutPoint, update: Option<&channelmonitor::ChannelMonitorUpdate>,
		data: &channelmonitor::ChannelMonitor<Signer>, update_id: MonitorUpdateId,
	) -> chain::ChannelMonitorUpdateStatus {

		if let Some(update) = update {
			// Track new counterparty commitment txs
			let commitment_transactions = data.counterparty_commitment_txs_from_update(update);
			let revokable_data = commitment_transactions.into_iter().filter_map( |txn| {  

				let trusted_transaction = txn.trust();
				let revokable_output = trusted_transaction.revokeable_output_index();
				let justice_transaction_output = trusted_transaction.build_to_local_justice_tx(trusted_transaction.feerate_per_kw() as u64,  data.get_funding_txo().1).ok().zip(revokable_output);

				justice_transaction_output.map(
					|(justice,output_index) | {
						let value = trusted_transaction.built_transaction().transaction.output[output_index].value;
						let signed_justice = data.sign_to_local_justice_tx(justice, 0, value , trusted_transaction.commitment_number()).ok();
						signed_justice.map(|txn| {
							RevokeableOutputData{
								commitment_txid: trusted_transaction.txid(),
								justice_transaction: txn
							}
						})
					}
				)	

			}).flatten().collect::<Vec<RevokeableOutputData>>();

            for justice_data in revokable_data {
                let result = self.add_update_appointment(justice_data);
                if result.is_err() {
                    // Todo Fix Return All
                    return chain::ChannelMonitorUpdateStatus::UnrecoverableError;
                }
            }
		}
        return chain::ChannelMonitorUpdateStatus::Completed;
	}

}