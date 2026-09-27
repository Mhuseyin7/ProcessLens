use crate::{ActionRisk, ProcessInfo};

pub fn classify_action_risk(process: &ProcessInfo, descendant_count: usize) -> ActionRisk {
    let name = process.name.to_lowercase();
    if process.pid <= 1
        || [
            "system",
            "systemd",
            "init",
            "wininit.exe",
            "csrss.exe",
            "smss.exe",
            "services.exe",
            "svchost.exe",
            "lsass.exe",
            "winlogon.exe",
            "dockerd",
            "com.docker.backend.exe",
        ]
        .contains(&name.as_str())
    {
        return ActionRisk::Protected;
    }
    if [
        "postgres",
        "postgres.exe",
        "mysqld",
        "redis-server",
        "mongod",
    ]
    .contains(&name.as_str())
        || descendant_count > 8
    {
        return ActionRisk::High;
    }
    if descendant_count > 0 {
        ActionRisk::Moderate
    } else {
        ActionRisk::Low
    }
}
