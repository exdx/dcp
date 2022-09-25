use anyhow::Result;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
    match dcp::config::get_args() {
        Err(e) => {
            error!("❌ error reading arguments {}", e);
            std::process::exit(1)
        }
        Ok(config) => dcp::run(config).await?,
    }

    Ok(())
}
