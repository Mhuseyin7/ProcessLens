use std::collections::HashMap;

use crate::{PortBinding, ProcessInfo, ServiceInfo, detect_project};

/// Groups visible port-owning processes by a detected project root. A process is
/// never attributed to a project unless its working directory proves the link.
pub fn group_development_services(
    processes: &[ProcessInfo],
    ports: &[PortBinding],
) -> Vec<ServiceInfo> {
    let by_pid: HashMap<u32, &ProcessInfo> = processes.iter().map(|p| (p.pid, p)).collect();
    let mut groups: HashMap<String, ServiceInfo> = HashMap::new();
    for port in ports {
        let Some(process) = port.pid.and_then(|pid| by_pid.get(&pid)).copied() else {
            continue;
        };
        let project = process
            .working_directory
            .value()
            .and_then(|cwd| detect_project(cwd));
        let key = project
            .as_ref()
            .map(|p| p.root.display().to_string())
            .unwrap_or_else(|| format!("pid:{}", process.pid));
        let name = project
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| process.name.clone());
        let entry = groups.entry(key).or_insert_with(|| ServiceInfo {
            name,
            runtime: project.as_ref().and_then(|p| p.runtime.clone()),
            framework: project.as_ref().and_then(|p| p.framework.clone()),
            project,
            processes: Vec::new(),
            ports: Vec::new(),
        });
        if !entry.processes.iter().any(|p| p.pid == process.pid) {
            entry.processes.push(process.clone());
        }
        entry.ports.push(port.clone());
    }
    let mut services: Vec<_> = groups.into_values().collect();
    services.sort_by(|a, b| a.name.cmp(&b.name));
    services
}
