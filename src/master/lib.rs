use crate::common::AppError;
use crate::master::MasterDataResponse;
use crate::query::master::MasterDataQuery;

pub async fn bulk_insert_masters(pool: &sqlx::MySqlPool) -> Result<(), AppError> {
    let path = std::env::var("MASTER_DATA_PATH").expect("MASTER_DATA_PATH must be set");
    let data = std::fs::read_to_string(&path)?;
    let root: MasterDataResponse = serde_json::from_str(&data)?;

    MasterDataQuery::replace_all(pool, &root).await?;

    println!("MasterData synced successfully.");
    Ok(())
}
