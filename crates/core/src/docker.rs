//! Docker Engine discovery through its supported local API. No Docker CLI output is parsed.

use bollard::{Docker, query_parameters::ListContainersOptionsBuilder};

use crate::DockerContainerInfo;

#[derive(Debug, thiserror::Error)]
pub enum DockerDiscoveryError {
    #[error("Docker Engine is unavailable: {0}")]
    Unavailable(#[from] bollard::errors::Error),
}

pub async fn list_running_containers() -> Result<Vec<DockerContainerInfo>, DockerDiscoveryError> {
    let docker = Docker::connect_with_local_defaults()?;
    let containers = docker
        .list_containers(Some(ListContainersOptionsBuilder::new().all(true).build()))
        .await?;
    Ok(containers
        .into_iter()
        .map(|container| {
            let labels = container.labels.unwrap_or_default();
            let published_ports = container
                .ports
                .unwrap_or_default()
                .into_iter()
                .filter_map(|port| port.public_port)
                .collect();
            DockerContainerInfo {
                id: container.id.unwrap_or_default(),
                name: container
                    .names
                    .and_then(|names| names.into_iter().next())
                    .unwrap_or_else(|| "unnamed".into())
                    .trim_start_matches('/')
                    .to_owned(),
                image: container.image.unwrap_or_else(|| "unknown".into()),
                compose_project: labels.get("com.docker.compose.project").cloned(),
                health: container.status.and_then(|status| {
                    status
                        .split_once("(healthy)")
                        .map(|_| "healthy".to_owned())
                        .or_else(|| {
                            status
                                .split_once("(unhealthy)")
                                .map(|_| "unhealthy".to_owned())
                        })
                }),
                published_ports,
            }
        })
        .collect())
}
