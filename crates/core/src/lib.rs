//! Shared, local-only domain and discovery logic used by ProcessLens interfaces.

mod docker;
mod inspect;
mod platform;
mod project;
mod risk;
mod service;
mod tree;
mod types;

pub use docker::{DockerDiscoveryError, list_running_containers};
pub use inspect::Inspector;
pub use platform::{PlatformNetworkProvider, PlatformPortProvider, SystemPortProvider};
pub use project::detect_project;
pub use risk::classify_action_risk;
pub use service::group_development_services;
pub use tree::build_process_tree;
pub use types::*;
