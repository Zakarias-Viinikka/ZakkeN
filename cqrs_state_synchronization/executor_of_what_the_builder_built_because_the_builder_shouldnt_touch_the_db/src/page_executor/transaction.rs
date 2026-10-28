use error_stuff::CqrsErr;
use web_internal_db::db_helper;

pub async fn run_in_transaction<Fut, F, T>(work: F) -> Result<T, CqrsErr>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, CqrsErr>>,
{
    let we_own = !db_helper::are_we_in_middle_of_transaction().await?.value;
    if we_own {
        db_helper::begin_all_or_nothing().await?;
    }
    match work().await {
        Ok(value) => {
            if we_own {
                db_helper::everything_went_perfectly().await?;
            }
            Ok(value)
        }
        Err(e) => {
            if we_own {
                db_helper::regret_everything().await?;
            }
            Err(e)
        }
    }
}
