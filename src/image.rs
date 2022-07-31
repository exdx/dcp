pub struct Image {
    pub image: String,
}

impl Image {
    pub fn split(&self) -> Option<(String, String)> {
        let image_split: Vec<&str> = self.image.split(":").collect();
        if image_split.len() == 0 {
            return None;
        }

        let repo: String;
        if let Some(i) = image_split.get(0) {
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

        return Some((repo, tag));
    }
}
