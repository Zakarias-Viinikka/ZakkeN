use my_yrs_lib::YrsError;
use protocol::error::DbError;

pub enum CqrsErr {
    YrsErrorContainer(YrsError),
    DbErrorContainer(DbError),
}

impl From<YrsError> for CqrsErr {
    fn from(e: YrsError) -> Self {
        CqrsErr::YrsErrorContainer(e)
    }
}

impl From<DbError> for CqrsErr {
    fn from(e: DbError) -> Self {
        CqrsErr::DbErrorContainer(e)
    }
}
