use anyhow::{Result, Error};
use rocket::{routes, Config, State};
use zcash_vote_server::{context::Context, db::create_schema};

#[rocket::get("/")]
fn index(context: &State<Context>) -> Result<String, String> {
    let r = || {
        let connection = context.pool.get()?;
        let n = connection.query_row("SELECT COUNT(*) FROM t", [], |r| r.get::<_, u32>(0))?;
        Ok::<_, Error>(n.to_string())
    };
    r().map_err(|e| e.to_string())
}

#[rocket::launch]
fn launch() -> _ {
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let config = Config::figment();

    // Extract custom values
    let db_path: String = config.extract_inner("custom.db_path").expect("db_path");
    let context = Context::new(db_path);
    let connection = context.pool.get().unwrap();
    create_schema(&connection).unwrap();

    rocket::custom(config)
        .manage(context)
        .mount("/", routes![index])
}
