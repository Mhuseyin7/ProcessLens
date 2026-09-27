use crate::ProjectInfo;
use std::{
    fs,
    path::{Path, PathBuf},
};

const MANIFESTS: &[&str] = &[
    "package.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lock",
    "pyproject.toml",
    "requirements.txt",
    "composer.json",
    "Cargo.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "docker-compose.yml",
    "compose.yml",
];

pub fn detect_project(start: &Path) -> Option<ProjectInfo> {
    let resolved = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let mut current = if resolved.is_file() {
        resolved.parent()?.to_path_buf()
    } else {
        resolved
    };
    loop {
        let found: Vec<String> = MANIFESTS
            .iter()
            .filter(|name| current.join(name).is_file())
            .map(|s| (*s).to_owned())
            .collect();
        if !found.is_empty() {
            return Some(from_manifests(current, found));
        }
        if !current.pop() {
            return None;
        }
    }
}
fn from_manifests(root: PathBuf, manifests: Vec<String>) -> ProjectInfo {
    let has = |name: &str| manifests.iter().any(|m| m == name);
    let (language, runtime) = if has("package.json") {
        (Some("JavaScript/TypeScript".into()), Some("Node.js".into()))
    } else if has("pyproject.toml") || has("requirements.txt") {
        (Some("Python".into()), Some("Python".into()))
    } else if has("Cargo.toml") {
        (Some("Rust".into()), Some("Rust".into()))
    } else if has("go.mod") {
        (Some("Go".into()), Some("Go".into()))
    } else if has("composer.json") {
        (Some("PHP".into()), Some("PHP".into()))
    } else {
        (None, None)
    };
    let package_manager = if has("pnpm-lock.yaml") {
        Some("pnpm".into())
    } else if has("yarn.lock") {
        Some("yarn".into())
    } else if has("bun.lock") {
        Some("bun".into())
    } else if has("package.json") {
        Some("npm".into())
    } else {
        None
    };
    let package = fs::read_to_string(root.join("package.json"))
        .unwrap_or_default()
        .to_lowercase();
    let framework = if package.contains("\"next\"") {
        Some("Next.js".into())
    } else if package.contains("\"vite\"") {
        Some("Vite".into())
    } else if package.contains("\"@nestjs") {
        Some("NestJS".into())
    } else if package.contains("\"astro\"") {
        Some("Astro".into())
    } else if package.contains("\"@sveltejs/kit\"") {
        Some("SvelteKit".into())
    } else {
        None
    };
    let name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_owned();
    ProjectInfo {
        root,
        name,
        language,
        runtime,
        framework,
        package_manager,
        manifests,
    }
}
