use anyhow::{anyhow, Result};
use async_trait::async_trait;

use super::docker::Image as DockerImage;
use super::podman::Image as PodmanImage;
use super::Runtime;

/// Container is a trait that defines the functionality of a container
/// to be used by dcp. It contains various methods that are required for
/// the process of copying files out of a container but leaves the physical
/// actions of accomplishing this to the implementation.
///
/// # Functions
///
/// * `pull` - Pulls the container's image. Accepts authentication and can ignore local images if `force` is set.
/// * `start` - Starts the container and returns the started container's ID if successful.
/// * `stop` - Stops the container.
/// * `copy_files` - Copies the files from the specified locations to the specified destination locally.
/// * `present_locally` - Checks to see if the image is already pulled locally.
#[async_trait]
pub trait Container {
    async fn pull(&self, username: String, password: String, force: bool) -> Result<()>;
    async fn start(&self) -> Result<String>;
    async fn stop(&self, id: String) -> Result<()>;
    async fn copy_files(
        &self,
        content_path: String,
        download_path: String,
        write_to_stdout: bool,
    ) -> Result<()>;
    async fn present_locally(&self) -> bool;
}

/// Returns a container with the provided image and runtime
///
/// # Arguments
///
/// * `image` - String representation of an image
/// * `runtime` - Runtime object from representing what this container will run on
pub fn new(image: String, runtime: Runtime) -> Result<Box<dyn Container>> {
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

    if let Some(docker) = runtime.docker {
        return Ok(Box::new(DockerImage {
            image,
            repo,
            tag,
            runtime: docker,
        }));
    }

    if let Some(podman) = runtime.podman {
        return Ok(Box::new(PodmanImage {
            image,
            repo,
            tag,
            runtime: podman,
        }));
    }

    Err(anyhow!("failed to determine proper runtime for image"))
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
