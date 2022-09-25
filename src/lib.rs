use anyhow::{anyhow, Result};
use clap::{App, Arg};

mod runtime;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub const VERSION: &str = "0.4.0";

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
    // Username for singing into a private registry
    username: String,
    // Password for signing into a private registry
    password: String,
    // Force a pull even if the image is present locally
    force_pull: bool,
    // Specify a custom socket to utilize for the runtime
    socket: String,
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
                .short("c")
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
            Arg::with_name("username")
                .value_name("USERNAME")
                .help("Username used for singing into a private registry.")
                .short("u")
                .long("username")
                .default_value(""),

        )
        .arg(
            Arg::with_name("password")
                .value_name("PASSWORD")
                .help("Password used for signing into a private registry. * WARNING *: Writing credentials to your terminal is risky. Be sure you are okay with them showing up in your history")
                .short("p")
                .long("password")
                .default_value(""),

        )
        .arg(
            Arg::with_name("log-level")
                .value_name("LOG-LEVEL")
                .help("What level of logs to output. Accepts: [info, debug, trace, error, warn]")
                .short("l")
                .long("log-level")
                .default_value("debug"),
        )
        .arg(
            Arg::with_name("force-pull")
                .value_name("FORCE-PULL")
                .help("Force a pull even if the image is present locally")
                .takes_value(false)
                .long("force-pull")
                .short("f")
        )
        .arg(
            Arg::with_name("socket")
                .value_name("SOCKET")
                .help("Specify a custom socket to utilize for the runtime")
                .long("socket")
                .short("s")
                .default_value(runtime::DEFAULT_SOCKET)
        )
        .get_matches();

    let image = matches.value_of("image").unwrap().to_string();
    let download_path = matches.value_of("download-path").unwrap().to_string();
    let content_path = matches.value_of("content-path").unwrap().to_string();
    let write_to_stdout = matches.is_present("write-to-stdout");
    let force_pull = matches.is_present("force-pull");
    let log_level = matches.value_of("log-level").unwrap().to_string();
    let socket = matches.value_of("socket").unwrap().to_string();
    // TODO (tyslaton): Need to come up with a way for this to be extracted from the docker config to be more secure locally.
    let username = matches.value_of("username").unwrap().to_string();
    let password = matches.value_of("password").unwrap().to_string();

    if write_to_stdout {
        return Err(anyhow!(
            "❌ error: writing to stdout is not currently implemented"
        ));
    };

    Ok(Config {
        image,
        download_path,
        content_path,
        write_to_stdout,
        log_level,
        username,
        password,
        force_pull,
        socket,
    })
}

/// Run runs a sequence of events with the provided image
/// Run supports copying container filesystems running on both the docker and podman runtimes
/// 1. Pull down the image
/// 2. Create a container, receiving the container id as a response
/// 3. Copy the container content to the specified directory
/// 4. Delete the container
pub async fn run(config: Config) -> Result<()> {
    pretty_env_logger::formatted_builder()
        .parse_filters(&config.log_level.clone())
        .init();

    // Build the runtime
    let rt = if let Some(runtime) = runtime::set(&config.socket).await {
        runtime
    } else {
        return Err(anyhow!("❌ no valid container runtime"));
    };

    // Build the image struct
    let container = match runtime::container::new(config.image, rt) {
        Ok(i) => i,
        Err(e) => {
            return Err(anyhow!("❌ error building the image: {}", e));
        }
    };

    // Pull the image
    match container
        .pull(config.username, config.password, config.force_pull)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow!("❌ error building the image: {}", e));
        }
    }

    // Copy files from the image
    match container
        .copy_files(
            config.content_path,
            config.download_path,
            config.write_to_stdout,
        )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow!("❌ error copying the image's files: {}", e));
        }
    }

    Ok(())
}
