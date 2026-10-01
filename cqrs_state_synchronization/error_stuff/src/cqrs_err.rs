use love_letter::BrokenHeart;
use my_yrs_lib::YrsError;
use protocol::error::DbError;

#[derive(Debug)]
pub enum CqrsErr {
    YrsErrorContainer(YrsError),
    DbErrorContainer(DbError),
    BrokenHeartContainer(BrokenHeart),
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

impl From<BrokenHeart> for CqrsErr {
    fn from(e: BrokenHeart) -> Self {
        CqrsErr::BrokenHeartContainer(e)
    }
}
