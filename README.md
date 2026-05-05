# Voor - A Personal Git Implementation in Rust
### Author:  
Arnau Muñoz Barrera  

> A personal, ground-up implementation of a distributed version control system inspired by Git, developed in Rust.  
> This project explores the internal mechanics of version control systems, from object storage and hashing to branching, checkout workflows, and remote synchronization.

---

## Project Overview

This project implements a simplified but functional version control system inspired by Git.  
The backend is written in **Rust** and focuses on object storage, repository state, branch management, synchronization, and CLI usability.

The current delivery now behaves more like a real installable tool:

- The CLI ships as a named binary: `voor`
- The command surface is managed through structured argument parsing
- The CLI supports `--help`, `--version`, and `-C <path>`
- Repository commands automatically discover `.voor` in parent folders
- Auth can be stored in a user-level config outside the repository
- The project includes a GitHub Actions pipeline for build validation and ZIP release packaging

---

## Download & Install

For the executable download flow, use:

- [docs/download.md](/abs/path/C:/dev/Git-Implementation-Rust/docs/download.md)
- [docs/executable-validation.md](/abs/path/C:/dev/Git-Implementation-Rust/docs/executable-validation.md)

The intended end-user flow is:

1. Download the correct ZIP asset from GitHub Releases
2. Extract `voor` or `voor.exe`
3. Move it to a directory on `PATH`
4. Run `voor --version`
5. Use `voor init`, `voor login`, `voor push`, `voor pull`, and `voor serve`

---

## Objectives
- OB0 - **Development of a version control software:** understand how distributed coordination systems manage local and remote state across shared workspaces while exploring a tailored alternative for personal use cases.
- OB1 - **Develop a functional system:** provide a working version-control workflow for local and remote collaborative environments.
- OB3 - **Develop an analytical interface:** support richer visual and operational understanding of repository state through backend and frontend integration.
- OB4 - **Provide relevant metrics:** expose repository activity and state transitions in a way that improves developer visibility and decision-making.
- OB5 - **Explore the viability of an alternative solution:** evaluate a proprietary implementation as a modern alternative to traditional version-control tools.

---

## System Architecture

### Backend

- Rust CLI and repository engine
- `.voor` local repository layout
- Object store for blobs, trees, and commits
- Remote synchronization endpoints
- Clerk JWT validation
- Supabase-backed persistence for remote metadata

### Frontend

- React application for remote repository visualization and monitoring

---

## Core Features

- Repository initialization and discovery
- Staging, commits, branches, checkout, status, diff
- Push, pull, remote bootstrap, and DB sync
- Cross-directory CLI usage through repository root discovery
- Global auth token storage for reusable CLI sessions
- Local filesystem locking and atomic writes for repo mutations
- Single binary runtime, including `voor serve`

---

## Tech Stack

- **Language:** Rust  
- **Database:** Supabase  
- **Frontend:** React  
- **HTTP / API:** Axum  
- **Auth:** Clerk JWT  

---

## Project Structure

```text
/
├── backend/
│   └── voor/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── api/
│           ├── cli/
│           ├── tests/
│           └── utils/
├── docs/
│   ├── download.md
│   ├── executable-validation.md
│   └── service-monitoring-workflow.md
├── frontend/
│   └── voor/
└── .github/
    └── workflows/
        └── voor-release.yml
```

---

## Development Build

```powershell
cd .\backend\voor\
cargo build --release --bin voor
```

The release binary is generated at:

- Windows: `backend\voor\target\release\voor.exe`
- Linux / macOS: `backend/voor/target/release/voor`

For development-only execution:

```powershell
cargo run --bin voor -- --help
cargo run --bin voor -- --version
```

---

## CLI Usage

### Local repository commands

```powershell
voor init
voor status
voor add .
voor commit -m "message"
voor branch
voor checkout -b feature-branch
voor diff <hash> <file_path>
voor hash-object -w <file_path>
voor cat-file -p <hash>
```

### Remote and auth commands

```powershell
voor remote http://localhost:3000
voor login <clerk_jwt>
voor init-remote
voor push
voor pull master
voor sync-db
voor logout
```

### Server mode

```powershell
voor serve
```

The server reads configuration from environment variables such as:

```powershell
$env:SUPABASE_URL = "<postgres_url>"
$env:CLERK_JWT_ISSUER = "https://<your-clerk-domain>"
$env:CLERK_JWKS_URL = "https://<your-clerk-domain>/.well-known/jwks.json"
$env:CLERK_JWT_AUDIENCE = "<optional_audience>"
$env:PORT = "3000"
```

---

## Auth

This project uses **Clerk** for authentication and **Supabase Postgres** for persistence.

The API accepts Clerk bearer tokens on protected routes. After validation, the backend uses the Clerk `sub` as the user identifier in Supabase and ignores client-supplied `user_id` fields.

The backend reads:

- `CLERK_JWT_ISSUER`
- `CLERK_JWKS_URL`
- `CLERK_JWT_AUDIENCE` as optional

The CLI can authenticate through either:

- `voor login <clerk_jwt>`
- `VOOR_AUTH_TOKEN`

Global config location:

- Windows: `%APPDATA%\voor\config.toml`
- macOS: `~/Library/Application Support/voor/config.toml`
- Linux: `~/.config/voor/config.toml`

---

## Concurrency & Locking

This project uses a repository-scoped lock and atomic file replacement for critical local mutations.

- Lock path: `.voor/locks/repo.lock`
- Lock wait timeout: `15 seconds`
- Poll interval: `100 ms`
- Stale lock TTL: `5 minutes`

Protected write paths include:

- `init`
- `add`
- `commit`
- branch create and delete
- `checkout`
- `init-remote`
- `push`
- `pull`
- `sync-db`
- server-side sync handlers

Critical repo files are written through temporary files and then moved atomically into place.

---

## CI/CD

The repository includes a GitHub Actions pipeline at `.github/workflows/voor-release.yml`.

The workflow:

- Runs `cargo fmt --check`
- Runs `cargo clippy --all-targets --all-features -- -D warnings`
- Runs `cargo test --all-targets`
- Builds the `voor` release binary for Linux, macOS, and Windows
- Packages downloadable ZIP release assets
- Publishes those assets on GitHub Releases for version tags

Expected release assets:

- `voor-windows-x86_64.zip`
- `voor-linux-x86_64.zip`
- `voor-macos-x86_64.zip`

---

## Brief Execution Plan

1. Build the binary from `backend/voor`
2. Let GitHub Actions package ZIP release assets
3. Download the correct asset for the target platform
4. Extract the executable into a directory on `PATH`
5. Run `voor --version`
6. Run `voor init`
7. Run `voor login`, `voor push`, and `voor pull` against a configured server

---

## Validation

For the step-by-step executable validation flow, use:

- [docs/executable-validation.md](/abs/path/C:/dev/Git-Implementation-Rust/docs/executable-validation.md)
