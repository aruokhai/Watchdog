use reqwest::blocking::{Response};
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use teos_common::receipts::AppointmentReceipt;
use teos_common::{net::{http::Endpoint, NetAddr}, TowerId, UserId};
use teos_common::appointment::{Appointment, Locator};
use teos_common::{cryptography, protos as common_msgs};
/// Represents a generic api response.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Response(T),
    Error(ApiError),
}

/// API errors that can be received when interacting with the tower. Error codes match `teos_common::errors`.
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub error: String,
    pub error_code: u8,
}

/// Errors related to the `add_appointment` requests to the tower.
#[derive(Debug)]
pub enum AddAppointmentError {
    RequestError(RequestError),
    ApiError(ApiError),
    //Todo Fix Signature
    SignatureError,
    ConversionError(String),
    Unexpected
}


/// Errors related to requests sent to the tower.
#[derive(Debug, PartialEq, Eq)]
pub enum RequestError {
    ConnectionError(String),
    DeserializeError(String),
    Unexpected(String),
}

impl RequestError {
    pub fn is_connection(&self) -> bool {
        matches!(self, RequestError::ConnectionError(_))
    }
}


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

pub fn add_update_appointment(
    tower_id: TowerId,
    tower_net_addr: &NetAddr,
    appointment: &Appointment,
    signature: &str,
) -> Result<(u32, AppointmentReceipt), AddAppointmentError> {
    log::debug!(
        "Sending appointment {} to tower {tower_id}",
        appointment.locator
    );
    let request_data = common_msgs::AddAppointmentRequest {
        appointment: Some(appointment.clone().into()),
        signature: signature.to_owned(),
    };
    let response = post_request(
        tower_net_addr,
        Endpoint::AddAppointment,
        &request_data,
    )
    .map_err(|err| AddAppointmentError::RequestError(err))
    .map(|response| response.json().map_err(
        |e|
        AddAppointmentError::ConversionError(format!("Unexpected response body. Error: {e}"))))??;

    match response {

        ApiResponse::Response::<common_msgs::AddAppointmentResponse>(r) => {
            let receipt = AppointmentReceipt::with_signature(
                signature.to_owned(),
                r.start_block,
                r.signature.clone(),
            );
            let recovered_id = TowerId(
                cryptography::recover_pk(&receipt.to_vec(), &receipt.signature().unwrap()).unwrap(),
            );
            if recovered_id == tower_id {
                Ok((r.available_slots, receipt))
            } else {
                Err(AddAppointmentError::SignatureError)
            }
        }
        _ => Err(AddAppointmentError::Unexpected),

    }
    

}




/// A generic function to send a request to a tower.
fn request<S: Serialize>(
    tower_net_addr: &NetAddr,
    endpoint: Endpoint,
    method: Method,
    data: Option<S>,
) -> Result<Response, RequestError> {
    let client = reqwest::blocking::Client::new();

    let mut request_builder = client.request(
        method,
        format!("{}{}", tower_net_addr.net_addr(), endpoint.path()),
    );

    if let Some(data) = data {
        request_builder = request_builder.json(&data);
    }

    request_builder.send().map_err(|e| {
        log::debug!("An error ocurred when sending data to the tower: {e}");
        if e.is_connect() | e.is_timeout() {
            RequestError::ConnectionError(
                "Cannot connect to the tower. Connection refused".to_owned(),
            )
        } else {
            RequestError::Unexpected("Unexpected error ocurred (see logs for more info)".to_owned())
        }
    })
}

pub fn post_request<S: Serialize>(
    tower_net_addr: &NetAddr,
    endpoint: Endpoint,
    data: S,
) -> Result<Response, RequestError> {
    request(tower_net_addr, endpoint, Method::POST, Some(data))
}

pub fn get_request(
    tower_net_addr: &NetAddr,
    endpoint: Endpoint,
) -> Result<Response, RequestError> {
    request::<()>(tower_net_addr, endpoint, Method::GET, None)
}



