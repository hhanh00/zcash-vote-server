use anyhow::{Error, Result};
use rocket::{figment::Figment, routes, Build, Config, Rocket, State};
use rocket_cors::CorsOptions;
use rusqlite::params;
use tendermint_abci::ServerBuilder;
use zcash_vote_server::{
    chain::VoteChain,
    context::Context,
    db::{create_schema, store_election},
    election::scan_data_dir,
    routes::{get_ballot_height, get_election_by_id, get_num_ballots, post_ballot},
};

#[rocket::get("/")]
fn index(context: &State<Context>) -> Result<String, String> {
    let r = || {
        let connection = context.pool.get()?;
        let n = connection.query_row("SELECT COUNT(*) FROM t", [], |r| r.get::<_, u32>(0))?;
        Ok::<_, Error>(n.to_string())
    };
    r().map_err(|e| e.to_string())
}

pub fn init_context(config: &Figment) -> Result<Context> {
    let data_path: String = config.extract_inner("custom.data_path")?;
    let db_path: String = config.extract_inner("custom.db_path")?;
    let cometbft_port: u16 = config.extract_inner("custom.cometbft_port")?;
    let context = Context::new(data_path, db_path, cometbft_port);
    Ok(context)
}

async fn rocket_build(config: Figment, context: Context) -> Rocket<Build> {
    let init = async {
        let elections = scan_data_dir(&context.data_path)?;
        tracing::info!("# elections = {}", elections.len());
        let connection = context.pool.get()?;
        connection.execute("UPDATE elections SET closed = TRUE", [])?;
        for e in elections.iter() {
            let connection = context.pool.get()?;
            let id_election = store_election(&connection, e, false)?;
            let cmx_root = e.cmx_frontier.as_ref().unwrap().root();
            let frontier = serde_json::to_string(&e.cmx_frontier)?;
            connection.execute(
                "INSERT INTO cmx_frontiers(election, height, frontier)
            VALUES (?1, 0, ?2) ON CONFLICT DO NOTHING",
                params![id_election, &frontier],
            )?;
            connection.execute(
                "INSERT INTO cmx_roots(election, height, hash)
            VALUES (?1, 0, ?2) ON CONFLICT DO NOTHING",
                params![id_election, &cmx_root],
            )?;
        }

        Ok::<_, Error>(context)
    };
    let context = init.await.unwrap();

    let cors = CorsOptions::default().to_cors().unwrap();

    rocket::custom(config).attach(cors).manage(context).mount(
        "/",
        routes![
            index,
            get_election_by_id,
            post_ballot,
            get_num_ballots,
            get_ballot_height
        ],
    )
}

#[rocket::main]
pub async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let config = Config::figment();
    let context = init_context(&config).unwrap();
    {
        let connection = context.pool.get().unwrap();
        create_schema(&connection).unwrap();
    }

    let (app, runner) = VoteChain::new(context.pool.get().unwrap());
    let server = ServerBuilder::new(1_000_000)
        .bind(format!("{}:{}", "127.0.0.1", context.comet_bft), app)
        .unwrap();
    std::thread::spawn(move || {
        let res = runner.run();
        println!("{:?}", res);
    });
    std::thread::spawn(move || server.listen().unwrap());

    rocket_build(config, context).await.launch().await.unwrap();
}
