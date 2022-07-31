use anyhow::{anyhow, Result};
use clap::{App, Arg};
use docker_api::api::{ContainerCreateOpts, PullOpts, RmContainerOpts};
use docker_api::Docker;
use futures_util::{StreamExt, TryStreamExt};
use std::path::PathBuf;
use tar::Archive;

mod image;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

const DOCKER_SOCKET: &str = "unix:///var/run/docker.sock";
pub const VERSION: &str = "0.2.1";

#[derive(Debug)]
pub struct Config {
    // List of images
    image: String,
    // Where the download files should be saved on the filesystem. Default "."
    download_path: String,
    // Where the content (files) are in the container filesystem. Default "/"
    content_path: String,
    // Option to write to stdout instead of the local filesystem.
    write_to_stdout: bool,
    // What level of logs to output
    log_level: String,
}

pub fn get_args() -> Result<Config> {
    let matches = App::new("dcp")
        .version(VERSION)
        .author("exdx")
        .about("docker cp made easy")
        .arg(
            Arg::with_name("image")
                .value_name("IMAGE")
                .help("Container image to extract content from")
                .required(true),
        )
        .arg(
            Arg::with_name("download-path")
                .value_name("DOWNLOAD-PATH")
                .help("Where the image contents should be saved on the filesystem")
                .default_value(".")
                .short("d")
                .long("download-path"),
        )
        .arg(
            Arg::with_name("content-path")
                .value_name("CONTENT-PATH")
                .help("Where in the container filesystem the content to extract is")
                .short("p")
                .default_value("/")
                .long("content-path"),
        )
        .arg(
            Arg::with_name("write-to-stdout")
                .value_name("WRITE-TO-STDOUT")
                .help("Whether to write to stdout instead of the filesystem")
                .takes_value(false)
                .short("w")
                .long("write-to-stdout"),
        )
        .arg(
            Arg::with_name("log-level")
                .value_name("LOG-LEVEL")
                .help("What level of logs to output. Accepts: [info, debug, trace, error, warn]")
                .short("l")
                .long("log-level")
                .default_value("debug"),
        )
        .get_matches();

    let image = matches.value_of("image").unwrap().to_string();
    let download_path = matches.value_of("download-path").unwrap().to_string();
    let content_path = matches.value_of("content-path").unwrap().to_string();
    let write_to_stdout = matches.is_present("write-to-stdout");
    let log_level = matches.value_of("log-level").unwrap().to_string();

    if write_to_stdout {
        return Err(anyhow!(
            "error: writing to stdout is not currently implemented"
        ));
    };

    Ok(Config {
        image,
        download_path,
        content_path,
        write_to_stdout,
        log_level,
    })
}

/// Run runs a sequence of events with the provided image
/// 1. Pull down the image
/// 2. Create a container, receiving the container id as a response
/// 3. Copy the container content to the specified directory
/// 4. Delete the container
pub async fn run(config: Config) -> Result<()> {
    pretty_env_logger::formatted_builder()
        .parse_filters(&config.log_level.clone())
        .init();

    let docker = Docker::new(DOCKER_SOCKET)?;

    let image = image::Image {
        image: config.image.clone(),
    };

    let repo: String;
    let tag: String;
    match image.split() {
        Some(image) => {
            repo = image.0;
            tag = image.1;
        }
        None => {
            return Err(anyhow!(
                "could not split provided image into repository and tag"
            ))
        }
    }

    let pull_opts = PullOpts::builder().image(repo).tag(tag).build();
    let images = docker.images();
    let mut stream = images.pull(&pull_opts);

    while let Some(pull_result) = stream.next().await {
        match pull_result {
            Ok(output) => {
                debug!("🔧 {:?}", output);
            }
            Err(e) => {
                error!("❌ {}", e);
            }
        }
    }

    // note(tflannag): Use a "dummy" command "FROM SCRATCH" container images.
    let cmd = vec![""];
    let create_opts = ContainerCreateOpts::builder(config.image.clone())
        .cmd(&cmd)
        .build();
    let container = docker.containers().create(&create_opts).await?;
    let id = container.id();
    debug!("📦 Created container with id: {:?}", id);

    let mut content_path = PathBuf::new();
    content_path.push(&config.content_path);

    let mut download_path = PathBuf::new();
    download_path.push(&config.download_path);

    let bytes = docker
        .containers()
        .get(&*id)
        .copy_from(&content_path)
        .try_concat()
        .await?;

    let mut archive = Archive::new(&bytes[..]);
    if config.write_to_stdout {
        unimplemented!()
    } else {
        archive.unpack(&download_path)?;
    }

    info!(
        "✅ Copied content to {} successfully",
        download_path.display()
    );

    let delete_opts = RmContainerOpts::builder().force(true).build();
    if let Err(e) = docker.containers().get(&*id).remove(&delete_opts).await {
        error!("❌ Error cleaning up container {}", e);
    }

    debug!("📦 Cleaned up container {:?} successfully", id);

    Ok(())
}
