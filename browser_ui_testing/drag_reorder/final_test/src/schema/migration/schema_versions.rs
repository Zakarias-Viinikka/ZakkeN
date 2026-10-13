#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SchemaVersion {
    Version0,
}

pub const CURRENT_VERSION: SchemaVersion = SchemaVersion::Version0;

impl SchemaVersion {
    pub fn to_int(self) -> i64 {
        match self {
            SchemaVersion::Version0 => 0,
        }
    }

    pub fn from_int(v: i64) -> Option<Self> {
        match v {
            0 => Some(SchemaVersion::Version0),
            _ => None,
        }
    }
}
