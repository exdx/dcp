#[tokio::main]
async fn main() -> dcp::DCPResult<()> {
    match dcp::get_args() {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
        Ok(config) => dcp::run(config).await?,
    }

    Ok(())
}
