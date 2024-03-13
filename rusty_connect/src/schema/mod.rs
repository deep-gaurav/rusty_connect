

use async_graphql::{Schema};



use self::{mutation::Mutation, query::Query, subscription::Subscription};

pub mod mutation;
pub mod query;
pub mod subscription;

pub type GQSchema = Schema<Query, Mutation, Subscription>;
