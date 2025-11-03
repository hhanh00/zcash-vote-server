use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub struct Context {
    pub data_path: String,
    pub db_path: String,
    pub comet_bft: u16,
    pub pool: SqlitePool,
}

impl Context {
    pub async fn new(data_path: String, db_path: String, comet_bft: u16) -> Self {
        let options = SqliteConnectOptions::new().filename(&db_path);
        let pool =
            SqlitePool::connect_with(options).await.unwrap();

        Self {
            data_path,
            db_path,
            comet_bft,
            pool,
        }
    }
}
