use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Availability<T> {
    Available(T),
    Unavailable { reason: String },
}

impl<T> Availability<T> {
    pub fn value(&self) -> Option<&T> {
        if let Self::Available(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub executable: Availability<PathBuf>,
    pub command: Availability<Vec<String>>,
    pub working_directory: Availability<PathBuf>,
    pub start_time_unix: u64,
    pub memory_bytes: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessTreeNode {
    pub process: ProcessInfo,
    pub children: Vec<ProcessTreeNode>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortBinding {
    pub protocol: Protocol,
    pub local_address: String,
    pub local_port: u16,
    pub state: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,
    pub language: Option<String>,
    pub runtime: Option<String>,
    pub framework: Option<String>,
    pub package_manager: Option<String>,
    pub manifests: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectionResult {
    pub binding: PortBinding,
    pub process: Option<ProcessInfo>,
    pub project: Option<ProjectInfo>,
    pub tree: Option<ProcessTreeNode>,
    pub risk: Option<ActionRisk>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ActionRisk {
    Protected,
    High,
    Moderate,
    Low,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub project: Option<ProjectInfo>,
    pub runtime: Option<String>,
    pub framework: Option<String>,
    pub processes: Vec<ProcessInfo>,
    pub ports: Vec<PortBinding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkConnection {
    pub protocol: Protocol,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub state: Option<String>,
    pub pid: Option<u32>,
    pub address_family: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DockerContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub compose_project: Option<String>,
    pub health: Option<String>,
    pub published_ports: Vec<u16>,
}
