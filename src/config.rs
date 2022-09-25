use anyhow::{anyhow, Result};
use clap::{App, Arg};

use crate::runtime;

pub const VERSION: &str = "0.4.0";

#[derive(Debug)]
pub struct Config {
    // List of images
    pub image: String,
    // Where the download files should be saved on the filesystem. Default "."
    pub download_path: String,
    // Where the content (files) are in the container filesystem. Default "/"
    pub content_path: String,
    // Option to write to stdout instead of the local filesystem.
    pub write_to_stdout: bool,
    // What level of logs to output
    pub log_level: String,
    // Username for singing into a private registry
    pub username: String,
    // Password for signing into a private registry
    pub password: String,
    // Force a pull even if the image is present locally
    pub force_pull: bool,
    // Specify a custom socket to utilize for the runtime
    pub socket: String,
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
        return Err(anyhow!("❌ writing to stdout is not currently implemented"));
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
