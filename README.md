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

## Install And Update Voor Core

### Windows

Build the release executable and copy it over the existing installed binary:

```powershell
cd .\backend\voor\
cargo build --release --bin voor
New-Item -ItemType Directory -Force "$env:LOCALAPPDATA\Programs\voor"
Copy-Item -LiteralPath ".\target\release\voor.exe" -Destination "$env:LOCALAPPDATA\Programs\voor\voor.exe" -Force
```

Add this folder to the user `PATH` if it is not already present:

```powershell
[Environment]::SetEnvironmentVariable(
  "Path",
  [Environment]::GetEnvironmentVariable("Path", "User") + ";$env:LOCALAPPDATA\Programs\voor",
  "User"
)
```

Open a new terminal and verify:

```powershell
voor --help
voor --version
```

To update Voor later, rebuild and run the same `Copy-Item` command. This overwrites the existing core executable in place.

### Linux / macOS

Build the release executable and install it into a directory on `PATH`:

```bash
cd backend/voor
cargo build --release --bin voor
mkdir -p ~/.local/bin
cp ./target/release/voor ~/.local/bin/voor
chmod +x ~/.local/bin/voor
```

Ensure `~/.local/bin` is on `PATH`, then verify:

```bash
voor --help
voor --version
```

To update Voor later, rebuild and run the same `cp` command. This overwrites the existing core executable in place.

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
voor login
voor init-remote
voor push
voor pull master
voor sync
voor sync-db
voor logout
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

- `voor login`, which opens the Clerk login/sign-up flow in the browser
- automatic browser login when `voor init-remote`, `voor push`, `voor pull`, or `voor sync-db` needs a token
- `VOOR_AUTH_TOKEN`

`voor init` writes `.voor/config` with `url = http://localhost:3000` by default, so the remote URL does not need to be set manually for local development.

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

## Releases

Expected release assets:

- `voor-windows-x86_64.zip`
- `voor-linux-x86_64.zip`
- `voor-macos-x86_64.zip`

 ---

## Public Demo Environment

A public demonstration environment is available for evaluation purposes.

The demo is deployed independently from the production platform and provides a frontend-only experience designed to showcase the main capabilities of Voor without exposing internal infrastructure, repositories, services, or user data.

### Purpose

The demo allows evaluators, recruiters, supervisors, and external reviewers to explore the platform UI and user experience without requiring access to:

- Production repositories
- Production databases
- Backend services
- Authentication credentials
- Repository synchronization features
- Administrative functionality

### Architecture

The demo is deployed as a standalone frontend application on Vercel and is completely isolated from the production environment.

Characteristics:

- No database connection
- No backend dependency
- No repository access
- No write operations
- No external service integrations
- No access to real user data

All information displayed within the demo is generated from predefined mock datasets.

### Mock Data

The demonstration environment includes realistic hardcoded data covering approximately three months of simulated repository activity.

The generated datasets include:

- Repository information
- Branches and branch metrics
- Commit histories
- Activity timelines
- Dashboard statistics
- Monitoring information
- Repository analytics
- User and organization data
- Charts and historical trends

All relationships between entities are intentionally maintained to provide a realistic representation of platform usage.

### Authentication

The demo environment uses a simplified authentication flow intended exclusively for evaluation purposes.

No production credentials are required and no authentication information is shared with the main platform.

### Limitations

The demo environment is intentionally restricted and does not execute real operations.

The following features are disabled:

- Repository creation
- Repository synchronization
- Push operations
- Pull operations
- Remote initialization
- Database modifications
- Background jobs
- Monitoring execution
- Administrative actions

Interactive elements remain available when necessary to preserve the user experience, but actions do not affect any real system state.

### Deployment

The public demo is deployed independently using Vercel and can be updated without impacting the production platform, backend services, or database infrastructure.

This separation ensures that demonstrations, evaluations, and academic reviews can be performed safely while preserving the integrity of the operational environment.
