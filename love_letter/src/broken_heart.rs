#[derive(Debug, uniffi::Error)]
pub enum BrokenHeart {
    Error(String),
}

impl std::fmt::Display for BrokenHeart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrokenHeart::Error(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for BrokenHeart {}

impl From<bincode::Error> for BrokenHeart {
    fn from(err: bincode::Error) -> Self {
        BrokenHeart::Error(err.to_string())
    }
}
