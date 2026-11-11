//! Caller pov: what the user decided for one mismatched page. `Mine`
//! sends the client's block list. `Server` sends nothing (the server
//! already has that state). `Merged` sends whatever the diff overlay
//! produced.

#[derive(Debug, Clone)]
pub enum ResolveChoice {
    Mine,
    Server,
    Merged(Vec<String>),
}

impl ResolveChoice {
    pub fn label(&self) -> &'static str {
        match self {
            ResolveChoice::Mine => "mine",
            ResolveChoice::Server => "server",
            ResolveChoice::Merged(_) => "merged",
        }
    }
}
