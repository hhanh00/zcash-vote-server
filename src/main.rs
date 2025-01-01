use anyhow::Result;

#[rocket::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
