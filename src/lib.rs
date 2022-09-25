use anyhow::{anyhow, Result};

pub mod config;
mod runtime;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

/// Run runs a sequence of events with the provided image
/// Run supports copying container filesystems running on both the docker and podman runtimes
/// 1. Pull down the image
/// 2. Create a container, receiving the container id as a response
/// 3. Copy the container content to the specified directory
/// 4. Delete the container
pub async fn run(cfg: config::Config) -> Result<()> {
    pretty_env_logger::formatted_builder()
        .parse_filters(&cfg.log_level.clone())
        .init();

    // Build the runtime
    let rt = if let Some(runtime) = runtime::set(&cfg.socket).await {
        runtime
    } else {
        return Err(anyhow!("❌ no valid container runtime"));
    };

    // Build the image struct
    let container = match runtime::container::new(cfg.image, rt) {
        Ok(i) => i,
        Err(e) => {
            return Err(anyhow!("❌ error building the image: {}", e));
        }
    };

    // Pull the image
    match container
        .pull(cfg.username, cfg.password, cfg.force_pull)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow!("❌ error building the image: {}", e));
        }
    }

    // Copy files from the image
    match container
        .copy_files(cfg.content_path, cfg.download_path, cfg.write_to_stdout)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow!("❌ error copying the image's files: {}", e));
        }
    }

    Ok(())
}
