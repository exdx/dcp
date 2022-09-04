use anyhow::{anyhow, Result};
use docker_api::Docker;
use podman_api::Podman;
use xdg::BaseDirectories;

pub struct Runtime {
    pub docker: Option<docker_api::Docker>,
    pub podman: Option<podman_api::Podman>,
}

pub const DEFAULT_SOCKET: &str = "unix:///var/run/docker.sock";

pub async fn set(socket: &str) -> Option<Runtime> {
    match Docker::new(socket) {
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
                    let podman_socket = get_podman_socket(socket).ok()?;
                    match Podman::new(podman_socket) {
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

fn get_podman_socket(socket: &str) -> Result<String> {
    let mut podman_socket = String::from(socket);

    // If a custom socket has not be set, find the logical default
    if socket == DEFAULT_SOCKET {
        let base_dirs = BaseDirectories::new()?;
        if !base_dirs.has_runtime_directory() {
            return Err(anyhow!("could not find xdg runtime directory"));
        }
        let runtime_dir = base_dirs.get_runtime_directory()?;
        podman_socket = format!(
            "{}{}{}",
            "unix://",
            runtime_dir.as_path().to_str().unwrap(),
            "/podman/podman.sock"
        );
    }

    debug!("🔧 podman socket at {}", podman_socket);

    Ok(podman_socket)
}
