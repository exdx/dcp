use anyhow::{anyhow, Result};
use async_trait::async_trait;

use super::docker::DockerImage;
use super::podman::PodmanImage;
use super::runtime;

#[async_trait]
pub trait Image {
    async fn pull(&self, username: String, password: String, force: bool) -> Result<()>;
    async fn start(&self) -> Result<String>;
    async fn stop(&self, id: String) -> Result<()>;
    async fn copy_files(
        &self,
        content_path: String,
        download_path: String,
        write_to_stdout: bool,
    ) -> Result<()>;
    async fn is_present_locally(&self) -> bool;
}

// new creates a new image struct based on the image and runtime provided.
pub fn new(image: String, runtime: runtime::Runtime) -> Result<Box<dyn Image>> {
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
