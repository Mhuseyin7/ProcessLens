use crate::{
    Availability, InspectionResult, NetworkConnection, PlatformNetworkProvider,
    PlatformPortProvider, ProcessInfo, ServiceInfo, SystemPortProvider, build_process_tree,
    classify_action_risk, detect_project, group_development_services,
};
use std::{collections::HashMap, path::Path};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

pub struct Inspector<P: PlatformPortProvider = SystemPortProvider> {
    ports: P,
}
impl Default for Inspector<SystemPortProvider> {
    fn default() -> Self {
        Self {
            ports: SystemPortProvider,
        }
    }
}
impl<P: PlatformPortProvider> Inspector<P> {
    pub fn new(ports: P) -> Self {
        Self { ports }
    }
    pub fn processes(&self) -> Vec<ProcessInfo> {
        let mut system = System::new();
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        system.processes().values().map(convert).collect()
    }
    pub fn ports(&self) -> std::io::Result<Vec<crate::PortBinding>> {
        self.ports.list_ports()
    }
    pub fn services(&self) -> std::io::Result<Vec<ServiceInfo>> {
        let processes = self.processes();
        let ports = self.ports()?;
        Ok(group_development_services(&processes, &ports))
    }
    pub fn inspect_port(&self, port: u16) -> std::io::Result<Vec<InspectionResult>> {
        let processes = self.processes();
        let by_pid: HashMap<_, _> = processes.iter().map(|p| (p.pid, p)).collect();
        Ok(self
            .ports()?
            .into_iter()
            .filter(|b| b.local_port == port)
            .map(|binding| {
                let process = binding
                    .pid
                    .and_then(|pid| by_pid.get(&pid))
                    .cloned()
                    .cloned();
                let project = process
                    .as_ref()
                    .and_then(|p| p.working_directory.value())
                    .and_then(|p| detect_project(Path::new(p)));
                let tree = process
                    .as_ref()
                    .and_then(|p| build_process_tree(&processes, p.pid));
                let descendants = tree.as_ref().map(count_descendants).unwrap_or(0);
                let risk = process
                    .as_ref()
                    .map(|p| classify_action_risk(p, descendants));
                InspectionResult {
                    binding,
                    process,
                    project,
                    tree,
                    risk,
                }
            })
            .collect())
    }
}
impl<P: PlatformPortProvider + PlatformNetworkProvider> Inspector<P> {
    pub fn network(&self) -> std::io::Result<Vec<NetworkConnection>> {
        self.ports.list_connections()
    }
}
fn convert(p: &sysinfo::Process) -> ProcessInfo {
    ProcessInfo {
        pid: p.pid().as_u32(),
        parent_pid: p.parent().map(Pid::as_u32),
        name: p.name().to_string_lossy().into_owned(),
        executable: p
            .exe()
            .map(|v| Availability::Available(v.to_path_buf()))
            .unwrap_or_else(|| Availability::Unavailable {
                reason: "not available (permission or platform limitation)".into(),
            }),
        command: if p.cmd().is_empty() {
            Availability::Unavailable {
                reason: "not available (permission or platform limitation)".into(),
            }
        } else {
            Availability::Available(
                p.cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect(),
            )
        },
        working_directory: p
            .cwd()
            .map(|v| Availability::Available(v.to_path_buf()))
            .unwrap_or_else(|| Availability::Unavailable {
                reason: "not available (permission or platform limitation)".into(),
            }),
        start_time_unix: p.start_time(),
        memory_bytes: p.memory(),
        status: format!("{:?}", p.status()),
    }
}
fn count_descendants(node: &crate::ProcessTreeNode) -> usize {
    node.children.iter().map(|c| 1 + count_descendants(c)).sum()
}
