# ProcessLens

> Developer workstation’ınızda çalışan process, port, network connection, project ve Docker service ilişkilerini yerelde inceleyen açık kaynak developer utility.

**ProcessLens**, `port → PID → process → project → runtime → framework → Docker container` ilişkisini görünür kılar. Generic bir Task Manager değildir; local development environment içindeki servislerin nereden çalıştığını ve neye ait olduğunu anlamak için tasarlanmıştır.

<p align="center">Open source olarak geliştirildi ve <a href="https://muhammedkoca.com.tr">muhammedkoca.com.tr</a> tarafından hazırlandı.</p>

---

## Neler yapar?

- `3000` portunu hangi process’in kullandığını bulur.
- Process’in PID, command, working directory ve parent/child tree bilgisini gösterir.
- `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod` gibi manifest’lerden project root algılar.
- Node.js, Python, Rust, Go ve PHP runtime’larını; güvenilir olduğunda Next.js, Vite, NestJS, Astro veya SvelteKit framework’lerini belirler.
- TCP/UDP port binding ve TCP network connection’larını listeler.
- Development service’leri project bazında gruplar.
- Docker Engine API üzerinden container, image, Compose project, health ve published port bilgilerini alır.
- Process termination öncesinde protected process, PID reuse ve descendant risk kontrolleri yapar.

Tüm inspection local machine üzerinde yapılır. Account, cloud service ve telemetry gerektirmez.

## Quick start

### Requirements

- Rust stable
- Windows 10/11 (mevcut first-class platform)
- Node.js 20+ — desktop app için
- Docker Desktop — Docker container inspection için isteğe bağlı

### CLI

```powershell
git clone https://github.com/Mhuseyin7/ProcessLens.git
cd ProcessLens
cargo run -p processlens -- doctor

# Port 3000'ü incele
cargo run -p processlens -- port 3000

# Tüm portları JSON formatında al
cargo run -p processlens -- ports --json
```

### Desktop app

```powershell
cd apps/desktop
npm install
npx tauri dev
```

Production frontend build için:

```powershell
npm run build
```

## CLI commands

| Command | Açıklama |
| --- | --- |
| `processlens ps` | Görünen process’leri listeler. |
| `processlens ports` | TCP/UDP binding’leri listeler. |
| `processlens port <port>` | Port owner, project, process tree ve Docker mapping gösterir. |
| `processlens pid <pid>` | Tek bir process’in metadata’sını gösterir. |
| `processlens tree <pid>` | Parent/child process tree gösterir. |
| `processlens network` | TCP connection’ları listeler. |
| `processlens project <path>` | Bir directory için project/runtime/framework detection çalıştırır. |
| `processlens projects` | Development service’leri project bazında gruplar. |
| `processlens docker` | Docker Engine API üzerinden container’ları listeler. |
| `processlens why <port>` | Port ownership için deterministic explanation üretir. |
| `processlens kill <pid> --yes` | Risk kontrolü sonrası process termination request gönderir. |
| `processlens doctor` | Platform capability durumunu gösterir. |

`ps`, `ports`, `port`, `pid`, `tree`, `network`, `project`, `projects` ve `docker` komutlarında `--json` kullanılabilir.

## Örnek: port inspection

```text
Port 3000

Status:
LISTENING

Process:
node.exe

PID:
18432

Working Directory:
C:\Projects\shop-platform

Detected Project:
shop-platform

Runtime:
Node.js

Framework:
Next.js
```

Bir published Docker port bulunduğunda container, image, Compose project ve health bilgisi aynı sonuçta eklenir.

## Desktop app

Tauri + React desktop app aşağıdaki view’ları içerir:

- Overview
- Projects
- Processes
- Ports
- Network
- Docker

Desktop ve CLI, duplicate discovery logic kullanmaz; ikisi de `processlens-core` Rust library üzerinden çalışır.

## Privacy & security

- Local-first: process ve project verisi dış servislere gönderilmez.
- Environment variable value’ları gösterilmez veya persist edilmez.
- Permission denied olan metadata, boş ya da uydurma değer yerine açıkça unavailable olarak gösterilir.
- Protected Windows process’leri (`svchost.exe`, `lsass.exe`, `services.exe` vb.) termination işleminden korunur.
- `kill` command’i explicit `--yes` ister ve işlem öncesinde PID identity’yi tekrar doğrular.
- Docker bilgisi Docker CLI parse edilerek değil, supported Docker Engine API ile alınır.

## Platform support

| Capability | Windows | Linux | macOS |
| --- | --- | --- | --- |
| Process discovery | ✅ | Architecture planned | Architecture planned |
| TCP/UDP ports | ✅ | Architecture planned | Architecture planned |
| TCP network | ✅ | Architecture planned | Architecture planned |
| Docker Engine API | ✅ | ✅ | ✅ |
| Desktop app | ✅ | Packaging planned | Packaging planned |

Linux/macOS adapter’ları ve packaging henüz release-ready değildir. ProcessLens bu alanlarda olmayan capability’leri varmış gibi göstermez.

## Development

```powershell
# Rust checks
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace

# Desktop frontend
cd apps/desktop
npm run build
```

GitHub Actions, Rust formatting/test/lint ve desktop frontend build adımlarını çalıştırır.

## Project structure

```text
crates/core/          Shared Rust domain and discovery logic
apps/cli/             Clap-based CLI
apps/desktop/         Tauri + React desktop application
.github/workflows/    CI workflow
docs/                 Product documentation (grows with the project)
```

## Contributing

Issue, improvement suggestion ve pull request’ler memnuniyetle karşılanır. Contribution öncesinde [CONTRIBUTING.md](CONTRIBUTING.md) dosyasındaki quality checks’i çalıştırın.

## License

Bu project [MIT License](LICENSE) ile lisanslanmıştır.

---

Built with care by [muhammedkoca.com.tr](https://muhammedkoca.com.tr) — open source, local-first, developer-focused.
