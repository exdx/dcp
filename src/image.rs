use docker_api::Images;
use podman_api::api::Images as PodmanImages;

pub struct Image {
    pub image: String,
}

impl Image {
    pub fn split(&self) -> Option<(String, String)> {
        let image_split: Vec<&str> = self.image.split(':').collect();
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
    match images.list(&Default::default()).await {
        Ok(images) => {
            for image in images {
                if let Some(repo_tag) = image.repo_tags {
                    for tag in repo_tag {
                        if tag == search_for_image {
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
    match images.list(&Default::default()).await {
        Ok(images) => {
            for image in images {
                if let Some(repo_tag) = image.repo_tags {
                    for tag in repo_tag {
                        if tag == search_for_image {
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
