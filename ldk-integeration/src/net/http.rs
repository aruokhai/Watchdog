// // use reqwest::{Method, Response};
// use serde::{de::DeserializeOwned, Deserialize, Serialize};
// use teos_common::{net::NetAddr, TowerId, UserId};


// /// Represents a generic api response.
// #[derive(Serialize, Deserialize, Debug)]
// #[serde(untagged)]
// pub enum ApiResponse<T> {
//     Response(T),
//     Error(ApiError),
// }

// /// API errors that can be received when interacting with the tower. Error codes match `teos_common::errors`.
// #[derive(Serialize, Deserialize, Debug)]
// pub struct ApiError {
//     pub error: String,
//     pub error_code: u8,
// }

// /// Errors related to requests sent to the tower.
// #[derive(Debug, PartialEq, Eq)]
// pub enum RequestError {
//     ConnectionError(String),
//     DeserializeError(String),
//     Unexpected(String),
// }

// impl RequestError {
//     pub fn is_connection(&self) -> bool {
//         matches!(self, RequestError::ConnectionError(_))
//     }
// }


// /// Handles the logic of interacting with the `register` endpoint of the tower.
// pub async fn register(
//     tower_id: TowerId,
//     user_id: UserId,
//     tower_net_addr: &NetAddr,
//     proxy: &Option<ProxyInfo>,
// ) -> Result<RegistrationReceipt, RequestError> {
//     log::info!("Registering in the Eye of Satoshi (tower_id={tower_id})");
//     process_post_response(
//         post_request(
//             tower_net_addr,
//             Endpoint::Register,
//             &common_msgs::RegisterRequest {
//                 user_id: user_id.to_vec(),
//             },
//             proxy,
//         )
//         .await,
//     )
//     .await
//     .map(|r: common_msgs::RegisterResponse| {
//         RegistrationReceipt::with_signature(
//             user_id,
//             r.available_slots,
//             r.subscription_start,
//             r.subscription_expiry,
//             r.subscription_signature,
//         )
//     })
// }

// /// A generic function to send a request to a tower.
// async fn request<S: Serialize>(
//     tower_net_addr: &NetAddr,
//     endpoint: Endpoint,
//     method: Method,
//     data: Option<S>,
// ) -> Result<Response, RequestError> {
//     let client = reqwest::Client::new()

//     let mut request_builder = client.request(
//         method,
//         format!("{}{}", tower_net_addr.net_addr(), endpoint.path()),
//     );

//     if let Some(data) = data {
//         request_builder = request_builder.json(&data);
//     }

//     request_builder.send().await.map_err(|e| {
//         log::debug!("An error ocurred when sending data to the tower: {e}");
//         if e.is_connect() | e.is_timeout() {
//             RequestError::ConnectionError(
//                 "Cannot connect to the tower. Connection refused".to_owned(),
//             )
//         } else {
//             RequestError::Unexpected("Unexpected error ocurred (see logs for more info)".to_owned())
//         }
//     })
// }

// pub async fn post_request<S: Serialize>(
//     tower_net_addr: &NetAddr,
//     endpoint: Endpoint,
//     data: S,
//     proxy: &Option<ProxyInfo>,
// ) -> Result<Response, RequestError> {
//     request(tower_net_addr, endpoint, proxy, Method::POST, Some(data)).await
// }

// pub async fn get_request(
//     tower_net_addr: &NetAddr,
//     endpoint: Endpoint,
//     proxy: &Option<ProxyInfo>,
// ) -> Result<Response, RequestError> {
//     request::<()>(tower_net_addr, endpoint, proxy, Method::GET, None).await
// }

// /// Generic function to process the response of a given post request.
// pub async fn process_post_response<T: DeserializeOwned>(
//     post_request: Result<Response, RequestError>,
// ) -> Result<T, RequestError> {
//     // TODO: Check if this can be switched for a map. Not sure how to handle async with maps
//     match post_request {
//         Ok(r) => r.json().await.map_err(|e| {
//             RequestError::DeserializeError(format!("Unexpected response body. Error: {e}"))
//         }),
//         Err(e) => Err(e),
//     }
// }