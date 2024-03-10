use std::collections::HashMap;

use all_devices::{DeviceFields, DeviceWithStateFields, DeviceWithStateFieldsDevice};
use graphql_client::GraphQLQuery;

#[allow(clippy::upper_case_acronyms)]
pub type JSON = serde_json::Value;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug,Clone,Serialize,Deserialize,PartialEq,Eq"
)]
pub struct AllDevices;

#[derive(Default)]
#[tauri_interop::emit_or_listen]
pub struct API {
    pub devices: Vec<DeviceWithStateFields>,
}

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "gql/queries.graphql",
    schema_path = "gql/schema.graphql",
    response_derives = "Debug"
)]
pub struct ConnectionSubscription;

