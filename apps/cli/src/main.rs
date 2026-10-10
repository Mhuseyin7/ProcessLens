use std::process::Command as OsCommand;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use processlens_core::{
    ActionRisk, Availability, DockerContainerInfo, InspectionResult, Inspector, ProcessInfo,
    ProcessTreeNode, build_process_tree, classify_action_risk, detect_project,
    list_running_containers,
};

#[derive(Parser)]
#[command(
    name = "processlens",
    version,
    about = "Local-first developer process and port inspection"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Ps {
        #[arg(long)]
        json: bool,
    },
    Ports {
        #[arg(long)]
        json: bool,
    },
    Port {
        port: u16,
        #[arg(long)]
        json: bool,
    },
    Pid {
        pid: u32,
        #[arg(long)]
        json: bool,
    },
    Tree {
        pid: u32,
        #[arg(long)]
        json: bool,
    },
    Network {
        #[arg(long)]
        json: bool,
    },
    Project {
        path: std::path::PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Active developer services, grouped only when a project root is detected.
    Projects {
        #[arg(long)]
        json: bool,
    },
    /// Docker Engine discovery is only reported when its native API adapter is available.
    Docker {
        #[arg(long)]
        json: bool,
    },
    Why {
        target: u16,
    },
    /// Explicitly confirmed, guarded process termination. Protected processes are refused.
    Kill {
        pid: u32,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        yes: bool,
    },
    Doctor,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .without_time()
        .init();
    let cli = Cli::parse();
    let inspector = Inspector::default();
    match cli.command {
        Command::Ps { json } => output(&inspector.processes(), json),
        Command::Ports { json } => output(&inspector.ports()?, json),
        Command::Port { port, json } => {
            let results = inspector.inspect_port(port)?;
            let docker = list_running_containers()
                .await
                .ok()
                .unwrap_or_default()
                .into_iter()
                .filter(|container| container.published_ports.contains(&port))
                .collect::<Vec<_>>();
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "bindings": results, "docker": docker })
                )
            } else {
                print_port_results(port, &results, &docker)
            }
        }
        Command::Pid { pid, json } => output(
            &inspector.processes().into_iter().find(|p| p.pid == pid),
            json,
        ),
        Command::Tree { pid, json } => {
            let processes = inspector.processes();
            let tree = build_process_tree(&processes, pid);
            if json {
                output(&tree, true)
            } else if let Some(tree) = tree {
                print_tree(&tree, "")
            } else {
                bail!("PID {pid} is not visible")
            }
        }
        Command::Network { json } => output(&inspector.network()?, json),
        Command::Project { path, json } => output(&detect_project(&path), json),
        Command::Projects { json } => output(&inspector.services()?, json),
        Command::Docker { json } => docker_status(json).await,
        Command::Why { target } => why(&inspector, target)?,
        Command::Kill { pid, force, yes } => terminate(&inspector, pid, force, yes)?,
        Command::Doctor => doctor(&inspector),
    }
    Ok(())
}

fn output<T: serde::Serialize + std::fmt::Debug>(value: &T, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(value).expect("serializable domain type")
        )
    } else {
        println!("{value:#?}")
    }
}

fn print_port_results(port: u16, results: &[InspectionResult], docker: &[DockerContainerInfo]) {
    if results.is_empty() && docker.is_empty() {
        println!("Port {port} has no visible TCP or UDP binding.");
        return;
    }
    for result in results {
        println!(
            "Port {}\n\nStatus:\n{}\n\nNetwork:\n{:?} {}:{}",
            port,
            result.binding.state,
            result.binding.protocol,
            result.binding.local_address,
            result.binding.local_port
        );
        if let Some(process) = &result.process {
            println!(
                "\nProcess:\n{}\n\nPID:\n{}\n\nCommand:\n{}\n\nWorking Directory:\n{}",
                process.name,
                process.pid,
                display_command(process),
                display_path(&process.working_directory)
            );
        } else {
            println!("\nProcess:\nUnavailable (the PID is no longer visible or access was denied)");
        }
        if let Some(project) = &result.project {
            println!(
                "\nDetected Project:\n{}\n\nRuntime:\n{}\n\nFramework:\n{}\n\nPackage Manager:\n{}",
                project.name,
                project.runtime.as_deref().unwrap_or("Unknown"),
                project
                    .framework
                    .as_deref()
                    .unwrap_or("Not confidently detected"),
                project.package_manager.as_deref().unwrap_or("Unknown")
            );
        }
        if let Some(tree) = &result.tree {
            println!("\nProcess Tree:");
            print_tree(tree, "");
        }
        if let Some(risk) = &result.risk {
            println!("\nTermination risk: {risk:?}");
        }
    }
    for container in docker {
        println!(
            "\nDocker owner:\n{}\n\nImage:\n{}\n\nCompose Project:\n{}\n\nHealth:\n{}",
            container.name,
            container.image,
            container.compose_project.as_deref().unwrap_or("Unknown"),
            container.health.as_deref().unwrap_or("Unknown")
        );
    }
}

fn print_tree(node: &ProcessTreeNode, prefix: &str) {
    println!("{prefix}{} ({})", node.process.name, node.process.pid);
    for (index, child) in node.children.iter().enumerate() {
        let last = index + 1 == node.children.len();
        let marker = if last { "└── " } else { "├── " };
        println!(
            "{prefix}{marker}{} ({})",
            child.process.name, child.process.pid
        );
        let child_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
        for grandchild in &child.children {
            print_tree(grandchild, &child_prefix);
        }
    }
}

fn why(inspector: &Inspector, port: u16) -> anyhow::Result<()> {
    let results = inspector.inspect_port(port)?;
    if results.is_empty() {
        println!("Port {port} has no visible TCP or UDP binding.");
        return Ok(());
    }
    for result in results {
        let process_name = result
            .process
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("an unavailable process");
        println!("Port {port} is occupied by {process_name}.");
        if let Some(project) = result.project.as_ref() {
            println!(
                "\nProject: {}\nDirectory: {}\nRuntime: {}",
                project.name,
                project.root.display(),
                project.runtime.as_deref().unwrap_or("Unknown")
            );
            if let Some(framework) = &project.framework {
                println!("Framework: {framework}");
            }
        }
        if let Some(process) = result.process.as_ref() {
            println!(
                "\nPID: {}\nCommand: {}",
                process.pid,
                display_command(process)
            );
        }
        println!(
            "\nRecommended actions:\n- inspect with `processlens port {port}`\n- open the detected project\n- terminate only if the process is no longer needed"
        )
    }
    Ok(())
}

fn terminate(inspector: &Inspector, pid: u32, force: bool, yes: bool) -> anyhow::Result<()> {
    let initial = inspector
        .processes()
        .into_iter()
        .find(|p| p.pid == pid)
        .context("PID is not visible")?;
    let descendants = inspector.processes();
    let descendant_count = build_process_tree(&descendants, pid)
        .map(|tree| count_descendants(&tree))
        .unwrap_or(0);
    let risk = classify_action_risk(&initial, descendant_count);
    if risk == ActionRisk::Protected {
        bail!(
            "refusing to terminate protected process {} ({pid})",
            initial.name
        );
    }
    if initial.start_time_unix == 0 {
        bail!(
            "refusing to terminate {} ({pid}) because process start time is unavailable; cannot safely rule out PID reuse",
            initial.name
        );
    }
    if !yes {
        bail!(
            "Refusing to terminate {} ({pid}) without --yes. Risk: {risk:?}. Re-run with --yes after review.",
            initial.name
        );
    }
    let revalidated = inspector
        .processes()
        .into_iter()
        .find(|p| p.pid == pid)
        .context("PID exited before termination")?;
    if revalidated.start_time_unix != initial.start_time_unix || revalidated.name != initial.name {
        bail!("PID identity changed; refusing termination to avoid a PID reuse race")
    }
    #[cfg(windows)]
    {
        let mut command = OsCommand::new("taskkill");
        command.args(["/PID", &pid.to_string()]);
        if force {
            command.arg("/F");
        }
        let result = command.output().context("could not launch taskkill")?;
        if !result.status.success() {
            bail!(
                "termination failed: {}",
                String::from_utf8_lossy(&result.stderr).trim()
            );
        }
        println!("Termination request sent for {} ({pid}).", initial.name);
    }
    #[cfg(not(windows))]
    {
        let _ = force;
        bail!("safe termination adapter is not implemented for this platform yet");
    }
    Ok(())
}

fn count_descendants(node: &ProcessTreeNode) -> usize {
    node.children
        .iter()
        .map(|child| 1 + count_descendants(child))
        .sum()
}
fn display_command(process: &ProcessInfo) -> String {
    match &process.command {
        Availability::Available(value) => value.join(" "),
        Availability::Unavailable { reason } => reason.clone(),
    }
}
fn display_path(value: &Availability<std::path::PathBuf>) -> String {
    match value {
        Availability::Available(value) => value.display().to_string(),
        Availability::Unavailable { reason } => reason.clone(),
    }
}

async fn docker_status(json: bool) {
    match list_running_containers().await {
        Ok(containers) => output(&containers, json),
        Err(error) if json => println!(
            "{}",
            serde_json::json!({ "available": false, "reason": error.to_string() })
        ),
        Err(error) => println!("{error}"),
    }
}
fn doctor(inspector: &Inspector) {
    println!(
        "Process discovery: {} processes visible",
        inspector.processes().len()
    );
    match inspector.ports() {
        Ok(ports) => println!("Port discovery: available ({} records)", ports.len()),
        Err(error) => println!("Port discovery: unavailable ({error})"),
    }
    match inspector.network() {
        Ok(connections) => println!(
            "Network discovery: available ({} TCP records)",
            connections.len()
        ),
        Err(error) => println!("Network discovery: unavailable ({error})"),
    }
    println!("Docker Engine API: run `processlens docker` to inspect availability");
}
