#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use processlens_core::{Inspector, list_running_containers};

#[tauri::command]
fn list_processes() -> Vec<processlens_core::ProcessInfo> { Inspector::default().processes() }
#[tauri::command]
fn list_ports() -> Result<Vec<processlens_core::PortBinding>, String> { Inspector::default().ports().map_err(|error| error.to_string()) }
#[tauri::command]
fn list_services() -> Result<Vec<processlens_core::ServiceInfo>, String> { Inspector::default().services().map_err(|error| error.to_string()) }
#[tauri::command]
fn list_network() -> Result<Vec<processlens_core::NetworkConnection>, String> { Inspector::default().network().map_err(|error| error.to_string()) }
#[tauri::command]
async fn list_docker() -> Result<Vec<processlens_core::DockerContainerInfo>, String> { list_running_containers().await.map_err(|error| error.to_string()) }

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_processes, list_ports, list_services, list_network, list_docker])
        .run(tauri::generate_context!())
        .expect("error while running ProcessLens desktop application");
}
