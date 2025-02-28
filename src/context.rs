use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub struct Context {
    pub data_path: String,
    pub db_path: String,
    pub comet_bft: u16,
    pub pool: Pool<SqliteConnectionManager>,
}

impl Context {
    pub fn new(data_path: String, db_path: String, comet_bft: u16) -> Self {
        let pool = Pool::new(SqliteConnectionManager::file(&db_path)).unwrap();

        Self {
            data_path,
            db_path,
            comet_bft,
            pool,
        }
    }
}
