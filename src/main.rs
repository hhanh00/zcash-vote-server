use anyhow::{Result, Error};
use rocket::{routes, Build, Config, Rocket, State};
use rocket_cors::CorsOptions;
use zcash_vote::download::download_reference_data;
use zcash_vote_server::{context::Context, db::{create_schema, store_cmx, store_cmx_root, store_election}, election::scan_data_dir, routes::{get_ballot_height, get_election_by_id, get_num_ballots, post_ballot}};

#[rocket::get("/")]
fn index(context: &State<Context>) -> Result<String, String> {
    let r = || {
        let connection = context.pool.get()?;
        let n = connection.query_row("SELECT COUNT(*) FROM t", [], |r| r.get::<_, u32>(0))?;
        Ok::<_, Error>(n.to_string())
    };
    r().map_err(|e| e.to_string())
}

async fn rocket_build() -> Rocket<Build> {
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let config = Config::figment();

    // Extract custom values
    let init = async {
        let data_path: String = config.extract_inner("custom.data_path")?;
        let db_path: String = config.extract_inner("custom.db_path")?;
        let context = Context::new(data_path, db_path);
        let elections = scan_data_dir(&context.data_path)?;
        tracing::info!("# elections = {}", elections.len());
        let connection = context.pool.get()?;
        create_schema(&connection)?;
        for e in elections.iter() {
            let connection = context.pool.get()?;
            let id_election = store_election(&connection, e)?;
            store_cmx_root(&connection, id_election, 0, &e.cmx.0)?;
        }

        Ok::<_, Error>(context)
    };
    let context = init.await.unwrap();

    let cors = CorsOptions::default()
    .to_cors()
    .unwrap();

    rocket::custom(config)
        .attach(cors)
        .manage(context)
        .mount("/", routes![index, get_election_by_id, post_ballot,
        get_num_ballots, get_ballot_height])
}

#[rocket::main]
pub async fn main() {
    rocket_build().await.launch().await.unwrap();
}
