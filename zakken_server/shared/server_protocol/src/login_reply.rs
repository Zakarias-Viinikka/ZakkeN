use serde::{Deserialize, Serialize};

use crate::drift_solution::DriftSolution;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LoginReply {
    pub drift_version: String,
    pub drift_reason: String,
    pub drift_solution: Option<DriftSolution>,
}
