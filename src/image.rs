use docker_api::Images;
use podman_api::api::Images as PodmanImages;

pub struct Image {
    pub image: String,
}

impl Image {
    pub fn split(&self) -> Option<(String, String)> {
        let image_split: Vec<&str> = if self.image.contains('@') {
            self.image.split('@').collect()
        } else {
            self.image.split(':').collect()
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
}

// TODO (tyslaton): Refactor image_present_locally functions to be a single function
// TODO: Refactor to pass image name as an option to list, instead of listing all images, if possible.
pub async fn is_present_locally_docker(images: Images, search_for_image: String) -> bool {
    debug!("📦 Searching for image {} locally", search_for_image);
    match images.list(&Default::default()).await {
        Ok(images) => {
            for image in images {
                if let Some(repo_tag) = image.repo_tags {
                    for tag in repo_tag {
                        if tag == search_for_image {
                            debug!("📦 Found image {} locally", search_for_image);
                            return true;
                        }
                    }
                }
            }
        }
        Err(e) => error!("❌ error occurred while searching for image locally: {}", e),
    }
    false
}

pub async fn is_present_locally_podman(images: PodmanImages, search_for_image: String) -> bool {
    debug!("📦 Searching for image {} locally", search_for_image);
    match images.list(&Default::default()).await {
        Ok(images) => {
            for image in images {
                if let Some(repo_tag) = image.repo_tags {
                    for tag in repo_tag {
                        if tag == search_for_image {
                            debug!("📦 Found image {} locally", search_for_image);
                            return true;
                        }
                    }
                }
            }
        }
        Err(e) => error!("❌ error occurred while searching for image locally: {}", e),
    }
    false
}

#[cfg(test)]
mod tests {
    use super::Image;

    #[test]
    fn test_split() {
        let digest_image = "quay.io/tflannag/bundles@sha256:145ccb5e7e73d4ae914160c066e49f35bc2be2bb86e4ab0002a802aa436599bf";
        let image = Image {
            image: digest_image.to_string(),
        };

        let (out_repo, out_tag) = image.split().unwrap();
        assert_eq!(out_repo, "quay.io/tflannag/bundles".to_string());
        assert_eq!(
            out_tag,
            "sha256:145ccb5e7e73d4ae914160c066e49f35bc2be2bb86e4ab0002a802aa436599bf".to_string()
        );
    }
}
