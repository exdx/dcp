use anyhow::{anyhow, Result};
use docker_api::Docker;
use podman_api::Podman;
use xdg::BaseDirectories;

const DOCKER_SOCKET: &str = "unix:///var/run/docker.sock";

pub struct Runtime {
    pub docker: Option<docker_api::Docker>,
    pub podman: Option<podman_api::Podman>,
}

pub async fn set() -> Option<Runtime> {
    match Docker::new(DOCKER_SOCKET) {
        Ok(docker) => {
            // Use version() as a proxy for socket connection status
            match docker.version().await {
                Ok(_) => Some(Runtime {
                    docker: Some(docker),
                    podman: None,
                }),
                Err(_) => {
                    // Fallback to podman config
                    debug!("🔧 docker socket not found: falling back to podman configuration");
                    let socket = get_podman_socket().ok()?;
                    match Podman::new(socket) {
                        // Use version() as a proxy for socket connection status
                        Ok(podman) => match podman.version().await {
                            Ok(_) => {
                                return Some(Runtime {
                                    docker: None,
                                    podman: Some(podman),
                                })
                            }
                            Err(_) => {
                                error!("❌ neither docker or podman sockets were found running on this host");
                                return None;
                            }
                        },
                        Err(_) => {
                            error!("❌ unable to create a podman client on the host");
                            return None;
                        }
                    }
                }
            }
        }
        Err(_) => {
            error!("❌ unable to create a docker client on the host");
            return None;
        }
    }
}

fn get_podman_socket() -> Result<String> {
    let base_dirs = BaseDirectories::new()?;
    if !base_dirs.has_runtime_directory() {
        return Err(anyhow!("could not find xdg runtime directory"));
    }
    let runtime_dir = base_dirs.get_runtime_directory()?;
    let podman_socket = format!(
        "{}{}{}",
        "unix://",
        runtime_dir.as_path().to_str().unwrap(),
        "/podman/podman.sock"
    );

    debug!("🔧 podman socket at {}", podman_socket);

    Ok(podman_socket)
}
