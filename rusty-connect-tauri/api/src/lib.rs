use std::{collections::HashMap, default};

use all_devices::{
    DeviceFields, DeviceWithStateFields, DeviceWithStateFieldsDevice, IdentityPayloadFields,
};
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};

#[allow(clippy::upper_case_acronyms)]
pub type JSON = serde_json::Value;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug,Clone,Serialize,Deserialize,PartialEq"
)]
pub struct AllDevices;

#[derive(Default)]
#[tauri_interop::emit_or_listen]
pub struct API {
    pub devices: Vec<DeviceWithStateFields>,
    pub event: KDEEvents,
    pub download_progress: Option<(
        String,
        IdentityPayloadFields,
        download_progress::DownloadProgressDownloadUpdate,
    )>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub enum KDEEvents {
    #[default]
    None,
    PairRequest(String),
}

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug"
)]
pub struct ConnectionSubscription;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug,Clone,Serialize, Deserialize"
)]
pub struct DownloadProgress;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug"
)]
pub struct BroadcastUdp;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug"
)]
pub struct Pair;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug"
)]
pub struct SendClipboard;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug"
)]
pub struct SendBattery;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug"
)]
pub struct SendPing;
