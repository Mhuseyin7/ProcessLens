import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import type { DockerContainerInfo, NetworkConnection, PortBinding, ProcessInfo, ServiceInfo } from "./types";
import "./styles.css";

const client = new QueryClient({ defaultOptions: { queries: { refetchInterval: 5000, retry: 1 } } });
const nav = ["Overview", "Projects", "Processes", "Ports", "Network", "Docker", "History", "Settings"];

function useLensQuery<T>(key: string, command: string) {
  return useQuery({ queryKey: [key], queryFn: () => invoke<T>(command) });
}
function App() {
  const [page, setPage] = useState("Overview");
  const [search, setSearch] = useState("");
  return <main className="shell"><aside><div className="brand">◉ ProcessLens</div><nav>{nav.map(item => <button className={page === item ? "active" : ""} key={item} onClick={() => setPage(item)}>{item}</button>)}</nav><p className="local">Local-first<br />No telemetry</p></aside><section className="content"><header><div><h1>{page}</h1><p>Developer processes, ports, projects and local services.</p></div><input aria-label="Search" value={search} onChange={event => setSearch(event.target.value)} placeholder="Search PID, port, project…" /></header><Page page={page} search={search} /></section></main>;
}
function Page({ page, search }: { page: string; search: string }) {
  if (page === "Overview" || page === "Projects") return <Services search={search} />;
  if (page === "Processes") return <Processes search={search} />;
  if (page === "Ports") return <Ports search={search} />;
  if (page === "Network") return <Network search={search} />;
  if (page === "Docker") return <Docker search={search} />;
  return <Empty page={page} />;
}
const matches = (value: unknown, query: string) => !query || JSON.stringify(value).toLowerCase().includes(query.toLowerCase());
function Services({ search }: { search: string }) {
  const query = useLensQuery<ServiceInfo[]>("services", "list_services");
  if (query.isLoading) return <Loading />; if (query.isError) return <Failure error={query.error} />;
  const services = (query.data ?? []).filter(service => matches(service, search));
  return <div className="grid">{services.map(service => <article className="card" key={`${service.name}-${service.processes[0]?.pid ?? "unknown"}`}><span className="eyebrow">{service.framework ?? service.runtime ?? "Process"}</span><h2>{service.name}</h2><p>{service.project?.root ?? "No project root reliably detected"}</p><div className="chips">{service.ports.map(port => <span key={`${port.protocol}-${port.local_port}`}>{port.protocol} {port.local_address}:{port.local_port}</span>)}</div><small>{service.processes.map(process => `${process.name} · PID ${process.pid}`).join(", ")}</small></article>)}</div>;
}
function Processes({ search }: { search: string }) { const query = useLensQuery<ProcessInfo[]>("processes", "list_processes"); if (query.isLoading) return <Loading />; if (query.isError) return <Failure error={query.error} />; return <Table headers={["Process", "PID", "Status", "Memory"]} rows={(query.data ?? []).filter(value => matches(value, search)).map(p => [p.name, p.pid, p.status, `${Math.round(p.memory_bytes / 1024 / 1024)} MB`])} />; }
function Ports({ search }: { search: string }) { const query = useLensQuery<PortBinding[]>("ports", "list_ports"); if (query.isLoading) return <Loading />; if (query.isError) return <Failure error={query.error} />; return <Table headers={["Protocol", "Address", "Port", "State", "PID"]} rows={(query.data ?? []).filter(value => matches(value, search)).map(p => [p.protocol, p.local_address, p.local_port, p.state, p.pid ?? "Unavailable"])} />; }
function Network({ search }: { search: string }) { const query = useLensQuery<NetworkConnection[]>("network", "list_network"); if (query.isLoading) return <Loading />; if (query.isError) return <Failure error={query.error} />; return <Table headers={["Local", "Remote", "State", "PID", "Family"]} rows={(query.data ?? []).filter(value => matches(value, search)).map(c => [c.local_endpoint, c.remote_endpoint, c.state ?? "—", c.pid ?? "Unavailable", c.address_family])} />; }
function Docker({ search }: { search: string }) { const query = useLensQuery<DockerContainerInfo[]>("docker", "list_docker"); if (query.isLoading) return <Loading />; if (query.isError) return <Failure error={query.error} />; return <Table headers={["Container", "Image", "Compose", "Health", "Published ports"]} rows={(query.data ?? []).filter(value => matches(value, search)).map(c => [c.name, c.image, c.compose_project ?? "—", c.health ?? "—", c.published_ports.join(", ") || "—"])} />; }
function Table({ headers, rows }: { headers: string[]; rows: (string | number)[][] }) { return <div className="table"><div className="row head">{headers.map(header => <strong key={header}>{header}</strong>)}</div>{rows.map((row, index) => <div className="row" key={index}>{row.map((cell, cellIndex) => <span key={cellIndex}>{cell}</span>)}</div>)}</div>; }
function Empty({ page }: { page: string }) { return <div className="empty"><h2>{page} is not available in this build</h2><p>ProcessLens does not simulate unsupported system data. Add the relevant local adapter before enabling this area.</p></div>; }
function Loading() { return <p className="muted">Refreshing local system data…</p>; }
function Failure({ error }: { error: unknown }) { return <div className="empty"><h2>Data unavailable</h2><p>{String(error)}</p></div>; }
createRoot(document.getElementById("root")!).render(<React.StrictMode><QueryClientProvider client={client}><App /></QueryClientProvider></React.StrictMode>);
