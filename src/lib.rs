use clap::{App, Arg};
use std::error::Error;

type DCPResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    // List of images
    images: Vec<String>,
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
            Arg::new("images")
                .name("IMAGES")
                .help("Container images to extract content from")
                .required(true)
                .multiple_values(true)
        )
        .arg(
            Arg::new("download_path")
                .name("DOWNLOAD_PATH")
                .help("Where the contents of the image should be saved on the filesystem")
                .default_value(".")
                .short('d')
                .long("download_path")
        )
        .arg(
            Arg::new("content_path")
                .name("CONTENT_PATH")
                .help("Where in the container filesystem the content to extract is")
                .short('p')
                .default_value("/")
                .long("content_path")
        )
        .arg(
            Arg::new("write_to_stdout")
                .name("WRITE_TO_STDOUT")
                .help("Whether to write to stdout instead of the filesystem")
                .takes_value(false)
                .short('w')
                .long("write_to_stdout")
        ).get_matches();

    let images = matches.values_of_lossy("images").unwrap();
    let download_path = matches.value_of("download_path").unwrap().to_string();
    let content_path = matches.value_of("content_path").unwrap().to_string();
    let write_to_stdout = matches.is_present("write_to_stdout");

    Ok(Config {
        images,
        download_path,
        content_path,
        write_to_stdout,
    })
}

pub fn run(config: Config) -> DCPResult<()> {
    println!("{:?}", config);
    Ok(())
}
