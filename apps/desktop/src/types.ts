export type Availability<T> = { Available: T } | { Unavailable: { reason: string } };
export interface ProcessInfo { pid: number; parent_pid: number | null; name: string; command: Availability<string[]>; working_directory: Availability<string>; start_time_unix: number; memory_bytes: number; status: string }
export interface PortBinding { protocol: "Tcp" | "Udp"; local_address: string; local_port: number; state: string; pid: number | null }
export interface ProjectInfo { root: string; name: string; runtime: string | null; framework: string | null; package_manager: string | null }
export interface ServiceInfo { name: string; project: ProjectInfo | null; runtime: string | null; framework: string | null; processes: ProcessInfo[]; ports: PortBinding[] }
export interface NetworkConnection { protocol: "Tcp" | "Udp"; local_endpoint: string; remote_endpoint: string; state: string | null; pid: number | null; address_family: string }
export interface DockerContainerInfo { id: string; name: string; image: string; compose_project: string | null; health: string | null; published_ports: number[] }
