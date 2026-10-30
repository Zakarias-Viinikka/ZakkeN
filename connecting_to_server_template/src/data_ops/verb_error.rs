#[derive(Debug)]
pub enum VerbError {
    Refused(String),
    Failed(String),
}

impl VerbError {
    pub fn refused(msg: impl Into<String>) -> Self {
        VerbError::Refused(msg.into())
    }
    pub fn failed(msg: impl Into<String>) -> Self {
        VerbError::Failed(msg.into())
    }
}

impl std::fmt::Display for VerbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerbError::Refused(m) | VerbError::Failed(m) => write!(f, "{m}"),
        }
    }
}

pub fn announce_verb_error(error: VerbError) {
    use crate::cool_popup::{show_popup_as, PopupKind};
    match error {
        VerbError::Refused(m) => show_popup_as(PopupKind::Warning, m),
        VerbError::Failed(m) => show_popup_as(PopupKind::Error, m),
    }
}
