use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub struct Context {
    pub db_path: String,
    pub pool: Pool<SqliteConnectionManager>,
}

impl Context {
    pub fn new(db_path: String) -> Self {
        let pool = Pool::new(SqliteConnectionManager::file(&db_path)).unwrap();

        Self {
            db_path,
            pool,
        }
    }
}
