use clap::{App, Arg};
use std::error::Error;
use docker_api::{Docker};
use docker_api::api::{ImageBuildChunk, PullOpts};
use docker_api::api::ImageBuildChunk::PullStatus;
use futures_util::StreamExt;

pub type DCPResult<T> = Result<T, Box<dyn Error>>;

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
}

pub fn get_args() -> DCPResult<Config> {
    let matches = App::new("dcp")
        .version("0.1.0")
        .author("exdx")
        .about("docker cp made easy")
        .arg(
            Arg::with_name("image")
                .value_name("IMAGE")
                .help("Container images to extract content from")
                .required(true)
        )
        .arg(
            Arg::with_name("download_path")
                .value_name("DOWNLOAD_PATH")
                .help("Where the contents of the image should be saved on the filesystem")
                .default_value(".")
                .short("d")
                .long("download_path")
        )
        .arg(
            Arg::with_name("content_path")
                .value_name("CONTENT_PATH")
                .help("Where in the container filesystem the content to extract is")
                .short("p")
                .default_value("/")
                .long("content_path")
        )
        .arg(
            Arg::with_name("write_to_stdout")
                .value_name("WRITE_TO_STDOUT")
                .help("Whether to write to stdout instead of the filesystem")
                .takes_value(false)
                .short("w")
                .long("write_to_stdout")
        ).get_matches();

    let image = matches.value_of("image");
    let download_path = matches.value_of("download_path").unwrap().to_string();
    let content_path = matches.value_of("content_path").unwrap().to_string();
    let write_to_stdout = matches.is_present("write_to_stdout");

    let mut img = String::new();
    if let Some(i) = image {
        img = i.to_string()
    }

    Ok(Config {
        image: img,
        download_path,
        content_path,
        write_to_stdout,
    })
}

pub async fn run(config: Config) -> DCPResult<()> {
    println!("{:#?}", config);
    let docker = Docker::new("unix:///var/run/docker.sock")?;
    let info = docker.info().await?;
    println!("{:#?}", info);

    let opts = PullOpts::builder().image(config.image).build();
    let images = docker.images();
    let mut stream = images.pull(&opts);

    while let Some(pull_result) = stream.next().await {
        match pull_result {
            Ok(output) => {
                println!("{:?}", output);
            },
            Err(e) => eprintln!("{}", e),
        }
    }


    Ok(())
}
