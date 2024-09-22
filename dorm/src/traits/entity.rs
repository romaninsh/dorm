use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Entity: Serialize + DeserializeOwned + Clone + Send + Sync + Sized + 'static {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmptyEntity {}

impl Entity for EmptyEntity {}
