use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConflictChoice {
    Server,
    Client,
}

pub type ReconcileDecisions = HashMap<String, ConflictChoice>;

#[derive(Clone, PartialEq, Debug)]
pub struct PageDiffSummary {
    pub page_id: String,
    pub client_title: String,
    pub server_title: String,
    pub only_client: usize,
    pub only_server: usize,
    pub changed: usize,
    pub moved: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ReconcilePlan {
    pub take_from_server: Vec<String>,
    pub push_to_server: Vec<String>,
    pub identical: Vec<String>,
    pub disable_locally: Vec<String>,
    pub disable_on_server: Vec<String>,
    pub conflicts: Vec<PageDiffSummary>,
}
