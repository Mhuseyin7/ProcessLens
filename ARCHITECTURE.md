# Architecture

`processlens-core` owns domain types and discovery orchestration. The CLI is a thin presenter; future Tauri commands must call the same core APIs.

Platform concerns are isolated behind `PlatformPortProvider`. Unsupported capabilities return an explicit error. Process metadata uses `Availability<T>` so a missing permission cannot be mistaken for an empty system value.

Termination is intentionally absent until process identity revalidation and platform-specific graceful-termination mechanisms are complete.
