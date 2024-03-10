use std::sync::Arc;

use async_graphql::{Object, Schema};

use crate::plugins::{clipboard::Clipboard, ping::Ping, PluginManager};

use self::{mutation::Mutation, query::Query, subscription::Subscription};

pub mod mutation;
pub mod query;
pub mod subscription;

pub type GQSchema = Schema<Query, Mutation, Subscription>;
