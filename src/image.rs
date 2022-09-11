use anyhow::{anyhow, Result};
use docker_api::api::{ContainerCreateOpts, PullOpts, RegistryAuth, RmContainerOpts};
use docker_api::Images;
use futures_util::{StreamExt, TryStreamExt};
use podman_api::api::Images as PodmanImages;
use podman_api::opts::ContainerCreateOpts as PodmanContainerCreateOpts;
use podman_api::opts::PullOpts as PodmanPullOpts;
use podman_api::opts::RegistryAuth as PodmanRegistryAuth;
use std::path::PathBuf;
use tar::Archive;

use super::runtime;

pub struct Image {
    pub image: String,
    pub repo: String,
    pub tag: String,
    pub runtime: runtime::Runtime,
}

// new creates a new image struct based on the image and runtime provided.
pub fn new(image: String, runtime: runtime::Runtime) -> Result<Image> {
    let repo: String;
    let tag: String;
    match split(image.clone()) {
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

    Ok(Image {
        image,
        repo,
        tag,
        runtime,
    })
}

impl Image {
    // pull ensures that the image is present locally and, if it is isn't
    // will do the work necessary to pull it.
    pub async fn pull(&self, username: String, password: String, force: bool) -> Result<()> {
        if let Some(docker) = &self.runtime.docker {
            let auth = RegistryAuth::builder()
                .username(username)
                .password(password)
                .build();

            let pull_opts = PullOpts::builder()
                .image(&self.repo)
                .tag(&self.tag)
                .auth(auth)
                .build();

            let present_locally = self.is_present_locally_docker(docker.images()).await;
            if force || !present_locally {
                let images = docker.images();
                let mut stream = images.pull(&pull_opts);
                while let Some(pull_result) = stream.next().await {
                    match pull_result {
                        Ok(output) => {
                            debug!("🔧 {:?}", output);
                        }
                        Err(e) => {
                            return Err(anyhow!("{}", e));
                        }
                    }
                }
                debug!("✅ Successfully pulled the image");
            } else {
                debug!("✅ Skipping the pull process as the image was found locally");
            }
        } else {
            let auth = PodmanRegistryAuth::builder()
                .username(username)
                .password(password)
                .build();
            let pull_opts = PodmanPullOpts::builder()
                .reference(self.image.clone().trim())
                .auth(auth)
                .build();
            let present_locally = self
                .is_present_locally_podman(self.runtime.podman.as_ref().unwrap().images())
                .await;
            if force || !present_locally {
                let images = self.runtime.podman.as_ref().unwrap().images();
                let mut stream = images.pull(&pull_opts);
                while let Some(pull_result) = stream.next().await {
                    match pull_result {
                        Ok(output) => {
                            debug!("🔧 {:?}", output);
                        }
                        Err(e) => {
                            return Err(anyhow!("{}", e));
                        }
                    }
                }
                debug!("✅ Successfully pulled the image");
            } else {
                debug!("✅ Skipping the pull process as the image was found locally");
            }
        }
        Ok(())
    }

    // copy_files uses the image_structs values to copy files from the
    // image's file systems appropriately.
    pub async fn copy_files(
        &self,
        content_path: String,
        download_path: String,
        write_to_stdout: bool,
    ) -> Result<()> {
        // Create the container
        let container_id = match self.start().await {
            Ok(id) => id,
            Err(e) => {
                return Err(anyhow!("failed to start the image: {}", e));
            }
        };

        let mut content_path_buffer = PathBuf::new();
        content_path_buffer.push(&content_path);

        let mut download_path_buffer = PathBuf::new();
        download_path_buffer.push(&download_path);

        // Get the files from the container
        let bytes;
        if let Some(docker) = &self.runtime.docker {
            bytes = docker
                .containers()
                .get(&*container_id)
                .copy_from(&content_path_buffer)
                .try_concat()
                .await?;
        } else {
            bytes = self
                .runtime
                .podman
                .as_ref()
                .unwrap()
                .containers()
                .get(&*container_id)
                .copy_from(&content_path_buffer)
                .try_concat()
                .await?;
        }

        // Unpack the archive
        let mut archive = Archive::new(&bytes[..]);
        if write_to_stdout {
            unimplemented!()
        } else {
            archive.unpack(&download_path_buffer)?;
        }

        info!(
            "✅ Copied content to {} successfully",
            download_path_buffer.display()
        );

        // Stop the container
        match self.stop(container_id).await {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow!("failed to stop the image: {}", e));
            }
        }

        Ok(())
    }

    // start takes the the image struct's values to build a container
    // by interacting the container runtime's socket.
    async fn start(&self) -> Result<String> {
        // note(tflannag): Use a "dummy" command "FROM SCRATCH" container images.
        let cmd = vec![""];
        let id;
        if let Some(docker) = &self.runtime.docker {
            let create_opts = ContainerCreateOpts::builder(&self.image).cmd(&cmd).build();
            let container = docker.containers().create(&create_opts).await?;
            id = container.id().to_string();
            debug!("📦 Created container with id: {:?}", id);
        } else {
            let create_opts = PodmanContainerCreateOpts::builder()
                .image(self.image.trim())
                .command(&cmd)
                .build();
            let container = self
                .runtime
                .podman
                .as_ref()
                .unwrap()
                .containers()
                .create(&create_opts)
                .await?;
            id = container.id;
            debug!("📦 Created container with id: {:?}", id);
        }
        Ok(id)
    }

    // stop takes the given container ID and interacts with the container
    // runtime socket to stop the container.
    async fn stop(&self, id: String) -> Result<()> {
        if let Some(docker) = &self.runtime.docker {
            let delete_opts = RmContainerOpts::builder().force(true).build();
            if let Err(e) = docker.containers().get(&*id).remove(&delete_opts).await {
                return Err(anyhow!("{}", e));
            }
            debug!("📦 Cleaned up container {:?} successfully", id);
            Ok(())
        } else {
            match self
                .runtime
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
                Err(e) => Err(anyhow!("failed to stop the image: {}", e)),
            }
        }
    }

    // TODO (tyslaton): Refactor image_present_locally functions to be a single function
    // TODO: Refactor to pass image name as an option to list, instead of listing all images, if possible.
    async fn is_present_locally_docker(&self, images: Images) -> bool {
        debug!("📦 Searching for image {} locally", self.image);
        match images.list(&Default::default()).await {
            Ok(images) => {
                for image in images {
                    if let Some(repo_tag) = image.repo_tags {
                        for tag in repo_tag {
                            if tag == self.image {
                                debug!("📦 Found image {} locally", self.image);
                                return true;
                            }
                        }
                    }
                }
            }
            Err(e) => error!("error occurred while searching for image locally: {}", e),
        }
        false
    }

    async fn is_present_locally_podman(&self, images: PodmanImages) -> bool {
        debug!("📦 Searching for image {} locally", self.image);
        match images.list(&Default::default()).await {
            Ok(images) => {
                for image in images {
                    if let Some(repo_tag) = image.repo_tags {
                        for tag in repo_tag {
                            if tag == self.image {
                                debug!("📦 Found image {} locally", self.image);
                                return true;
                            }
                        }
                    }
                }
            }
            Err(e) => error!("error occurred while searching for image locally: {}", e),
        }
        false
    }
}

fn split(image: String) -> Option<(String, String)> {
    let image_split: Vec<&str> = if image.contains('@') {
        image.split('@').collect()
    } else {
        image.split(':').collect()
    };

    if image_split.is_empty() {
        return None;
    }

    let repo: String;
    if let Some(i) = image_split.first() {
        repo = i.to_string();
    } else {
        return None;
    }

    let tag: String;
    if let Some(i) = image_split.get(1) {
        tag = i.to_string();
    } else {
        // Fall back to latest tag if none is provided
        tag = String::from("latest");
    }

    Some((repo, tag))
}
#[cfg(test)]
mod tests {
    use super::split;

    #[test]
    fn test_split() {
        let digest_image = "quay.io/tflannag/bundles@sha256:145ccb5e7e73d4ae914160c066e49f35bc2be2bb86e4ab0002a802aa436599bf";
        let image = digest_image.to_string();

        let (out_repo, out_tag) = split(image).unwrap();
        assert_eq!(out_repo, "quay.io/tflannag/bundles".to_string());
        assert_eq!(
            out_tag,
            "sha256:145ccb5e7e73d4ae914160c066e49f35bc2be2bb86e4ab0002a802aa436599bf".to_string()
        );
    }
}
