mod docker;
mod podman;

pub mod container;

use docker_api::Docker;

// Imports not used by windows environments
#[cfg(not(target_os = "windows"))]
use anyhow::anyhow;
#[cfg(not(target_os = "windows"))]
use anyhow::Result;
#[cfg(not(target_os = "windows"))]
use podman_api::Podman;

// Imports that cannot be used in windows environments
#[cfg(not(target_os = "windows"))]
use xdg::BaseDirectories;

// Set logical socket default per environment
#[cfg(not(target_os = "windows"))]
pub const DEFAULT_SOCKET: &str = "unix:///var/run/docker.sock";
#[cfg(target_os = "windows")]
pub const DEFAULT_SOCKET: &str = "tcp://localhost:2375";

pub struct Runtime {
    pub docker: Option<docker_api::Docker>,
    pub podman: Option<podman_api::Podman>,
}

pub async fn set(socket: &str) -> Option<Runtime> {
    match Docker::new(socket) {
        Ok(docker) => {
            // Use version() as a proxy for socket connection status
            match docker.version().await {
                Ok(_) => Some(Runtime {
                    docker: Some(docker),
                    podman: None,
                }),
                #[cfg(not(target_os = "windows"))]
                Err(_) => {
                    // Fallback to podman config
                    debug!("🔧 docker socket not found: falling back to podman configuration");
                    let podman_socket = get_podman_socket(socket).ok()?;
                    match Podman::new(podman_socket) {
                        // Use version() as a proxy for socket connection status
                        Ok(podman) => match podman.version().await {
                            Ok(_) => Some(Runtime {
                                docker: None,
                                podman: Some(podman),
                            }),
                            Err(err) => {
                                error!("❌ neither docker or podman sockets were found running at {} on this host: {}", socket, err);
                                None
                            }
                        },
                        Err(err) => {
                            error!("❌ unable to create a podman client on the host: {}", err);
                            None
                        }
                    }
                }
                #[cfg(target_os = "windows")]
                Err(err) => {
                    error!(
                        "❌ docker socket was not found running at {} on this host: {}",
                        socket, err
                    );
                    return None;
                }
            }
        }
        Err(err) => {
            error!("❌ unable to create a docker client on the host: {}", err);
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_podman_socket(socket: &str) -> Result<String> {
    let mut podman_socket = String::from(socket);

    // If a custom socket has not been set, find the logical default
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
