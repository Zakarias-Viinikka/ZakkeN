use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LoginReply {
    pub drift_version: String,
    pub drift_reason: String,
}
