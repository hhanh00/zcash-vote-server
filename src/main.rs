use anyhow::{Result, Error};
use rocket::{routes, Config, State};
use zcash_vote_server::{context::Context, db::{create_schema, store_election}, election::scan_data_dir, routes::get_election};

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
    let init = || {
        let data_path: String = config.extract_inner("custom.data_path")?;
        let db_path: String = config.extract_inner("custom.db_path")?;
        let context = Context::new(data_path, db_path);
        let elections = scan_data_dir(&context.data_path)?;
        tracing::info!("# elections = {}", elections.len());
        let connection = context.pool.get()?;
        create_schema(&connection)?;
        for e in elections.iter() {
            store_election(&connection, e)?;
        }

        Ok::<_, Error>(context)
    };
    let context = init().unwrap();

    rocket::custom(config)
        .manage(context)
        .mount("/", routes![index, get_election])
}
