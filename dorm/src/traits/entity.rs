use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Entity:
    Serialize + DeserializeOwned + Default + Clone + Send + Sync + Sized + 'static
{
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EmptyEntity {}

impl Entity for EmptyEntity {}
