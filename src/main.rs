use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    match dcp::get_args() {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
        Ok(config) => dcp::run(config).await?,
    }

    Ok(())
}
