use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use podman_api::opts::{ContainerCreateOpts, PullOpts, RegistryAuth};
use std::path::PathBuf;
use tar::Archive;

use super::container::Container;

pub struct Image {
    pub image: String,
    pub repo: String,
    pub tag: String,
    pub runtime: podman_api::Podman,
}

#[async_trait]
impl Container for Image {
    // pull ensures that the image is present locally and, if it is isn't
    // will do the work necessary to pull it.
    async fn pull(&self, username: String, password: String, force: bool) -> Result<()> {
        if self.present_locally().await {
            if !force {
                debug!("✅ Skipping the pull process as the image was found locally");
                return Ok(());
            }
            debug!("🔧 Force was set, ignoring images present locally")
        }

        let auth = RegistryAuth::builder()
            .username(username)
            .password(password)
            .build();
        let pull_opts = PullOpts::builder()
            .reference(self.image.clone().trim())
            .auth(auth)
            .build();

        let images = self.runtime.images();
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

        Ok(())
    }

    // copy_files uses the image_structs values to copy files from the
    // image's file systems appropriately.
    async fn copy_files(
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
        let bytes = self
            .runtime
            .containers()
            .get(&*container_id)
            .copy_from(&content_path_buffer)
            .try_concat()
            .await?;

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
        let create_opts = ContainerCreateOpts::builder()
            .image(self.image.trim())
            .command(&cmd)
            .build();
        let container = self.runtime.containers().create(&create_opts).await?;
        let id = container.id;

        debug!("📦 Created container with id: {:?}", id);
        Ok(id)
    }

    // stop takes the given container ID and interacts with the container
    // runtime socket to stop the container.
    async fn stop(&self, id: String) -> Result<()> {
        match self.runtime.containers().prune(&Default::default()).await {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow!("failed to stop the image: {}", e));
            }
        }
        debug!("📦 Cleaned up container {:?} successfully", id);
        Ok(())
    }

    async fn present_locally(&self) -> bool {
        debug!("📦 Searching for image {} locally", self.image);
        match self.runtime.images().list(&Default::default()).await {
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
