use anyhow::{anyhow, Result};
use clap::{App, Arg};
use docker_api::api::{ContainerCreateOpts, PullOpts, RegistryAuth, RmContainerOpts};
use futures_util::{StreamExt, TryStreamExt};
use podman_api::opts::ContainerCreateOpts as PodmanContainerCreateOpts;
use podman_api::opts::PullOpts as PodmanPullOpts;
use podman_api::opts::RegistryAuth as PodmanRegistryAuth;
use std::path::PathBuf;
use tar::Archive;

mod image;
mod runtime;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub const VERSION: &str = "0.3.2";

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
        .get_matches();

    let image = matches.value_of("image").unwrap().to_string();
    let download_path = matches.value_of("download-path").unwrap().to_string();
    let content_path = matches.value_of("content-path").unwrap().to_string();
    let write_to_stdout = matches.is_present("write-to-stdout");
    let force_pull = matches.is_present("force-pull");
    let log_level = matches.value_of("log-level").unwrap().to_string();
    // TODO (tyslaton): Need to come up with a way for this to be extracted from the docker config to be more secure locally.
    let username = matches.value_of("username").unwrap().to_string();
    let password = matches.value_of("password").unwrap().to_string();

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
        username,
        password,
        force_pull,
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

    let rt = if let Some(runtime) = runtime::set().await {
        runtime
    } else {
        return Err(anyhow!("no valid container runtime"));
    };

    if let Some(docker) = &rt.docker {
        let auth = RegistryAuth::builder()
            .username(config.username)
            .password(config.password)
            .build();
        let pull_opts = PullOpts::builder().image(repo).tag(tag).auth(auth).build();

        let present_locally = image::is_present_locally_docker(docker.images(), image.image).await;
        if config.force_pull || !present_locally {
            let images = docker.images();
            let mut stream = images.pull(&pull_opts);
            while let Some(pull_result) = stream.next().await {
                match pull_result {
                    Ok(output) => {
                        debug!("🔧 {:?}", output);
                    }
                    Err(e) => {
                        return Err(anyhow!("❌ error pulling image: {}", e));
                    }
                }
            }
        } else {
            debug!("✅ Skipping the pull process as the image was found locally");
        }
    } else {
        let auth = PodmanRegistryAuth::builder()
            .username(config.username)
            .password(config.password)
            .build();
        let pull_opts = PodmanPullOpts::builder()
            .reference(config.image.clone().trim())
            .auth(auth)
            .build();

        let present_locally =
            image::is_present_locally_podman(rt.podman.as_ref().unwrap().images(), image.image)
                .await;
        if config.force_pull || !present_locally {
            let images = rt.podman.as_ref().unwrap().images();
            let mut stream = images.pull(&pull_opts);
            while let Some(pull_result) = stream.next().await {
                match pull_result {
                    Ok(output) => {
                        debug!("🔧 {:?}", output);
                    }
                    Err(e) => {
                        return Err(anyhow!("❌ error pulling image: {}", e));
                    }
                }
            }
        } else {
            debug!("✅ Skipping the pull process as the image was found locally");
        }
    }

    // note(tflannag): Use a "dummy" command "FROM SCRATCH" container images.
    let cmd = vec![""];
    let id;
    if let Some(docker) = &rt.docker {
        let create_opts = ContainerCreateOpts::builder(config.image.clone())
            .cmd(&cmd)
            .build();
        let container = docker.containers().create(&create_opts).await?;
        id = container.id().to_string();
        debug!("📦 Created container with id: {:?}", id);
    } else {
        let create_opts = PodmanContainerCreateOpts::builder()
            .image(config.image.clone().trim())
            .command(&cmd)
            .build();
        let container = rt
            .podman
            .as_ref()
            .unwrap()
            .containers()
            .create(&create_opts)
            .await?;
        id = container.id;
        debug!("📦 Created container with id: {:?}", id);
    }

    let mut content_path = PathBuf::new();
    content_path.push(&config.content_path);

    let mut download_path = PathBuf::new();
    download_path.push(&config.download_path);

    let bytes;
    if let Some(docker) = &rt.docker {
        bytes = docker
            .containers()
            .get(&*id)
            .copy_from(&content_path)
            .try_concat()
            .await?;
    } else {
        bytes = rt
            .podman
            .as_ref()
            .unwrap()
            .containers()
            .get(&*id)
            .copy_from(&content_path)
            .try_concat()
            .await?;
    }

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

    if let Some(docker) = &rt.docker {
        let delete_opts = RmContainerOpts::builder().force(true).build();
        if let Err(e) = docker.containers().get(&*id).remove(&delete_opts).await {
            error!("❌ error cleaning up container {}", e);
            Ok(())
        } else {
            debug!("📦 Cleaned up container {:?} successfully", id);
            Ok(())
        }
    } else {
        match rt
            .podman
            .as_ref()
            .unwrap()
            .containers()
            .prune(&Default::default())
            .await
        {
            Ok(_) => {
                debug!("📦 Cleaned up container {:?} successfully", id);
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Ok(())
            }
        }
    }
}
