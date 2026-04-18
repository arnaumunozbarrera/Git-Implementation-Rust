# Git - A Personal Git Implementation in Rust
### Author:  
Arnau Muñoz Barrera  

> A personal, ground-up implementation of a distributed version control system inspired by Git, developed in Rust.  
> This project explores the internal mechanics of version control systems, from object storage and hashing to branching, merging, and remote synchronization.

---

## Project Overview

This project aims to design and implement a simplified but functional version control system inspired by Git.  
The system is developed entirely in **Rust**, focusing on performance, memory safety, and low-level system design.

The implementation is divided into two main components:

- A **backend core engine** responsible for repository logic, object storage, hashing, commits, branches, merges, and remote operations.
- A **frontend interface** (CLI and/or GUI) that interacts with the backend and exposes user-facing commands.

The goal is not to replicate Git entirely, but to deeply understand and re-engineer its architecture and internal workflows.

---

## Objectives
- OB0 - **Development of a version control software:** in order to gain a deeper understanding of how coordination systems operate and manage simultaneous, collaborative work across shared workspaces, with the aim of offering a more tailored alternative for specific personal use cases.
- OB1 - **Develop a functional system:** a software system that enables the execution of version-control operations in collaborative environments, both in local and remote workspaces.
- OB3 - **Develop an analytical interface:** design and build a system focused on analytically visualising repository status through dashboards and visual elements, enabling an intuitive understanding between the user and the system.
- OB4 - **Provide relevant metrics:** provide the user with relevant metrics and indicative, dynamic visualisations that improve both capability and ease of decision-making regarding project evolution and repository activity.
- OB5 - **Explore the viability of an alternative solution:** create a proprietary implementation as an alternative to traditional version-control systems, from a modern technical and architectural perspective, for a more user-adapted experience.

---

## System Architecture

### High-Level Architecture

-  

### Backend

-  

### Frontend

-  

---

## Core Features

-  Repository descovery
-  Command line
-  Future: remote repos
-  Future: API exposure
-  Future: Metrics
-  Future: Analytic UI / Dashboards

---

## Tech Stack

- **Language:** Rust  
- **Database:** Supabase
- **Frontend:** React

---

## Project Structure

```
/
├── backend/        # Core VCS engine
│   ├── personal-git/src/
│   │    ├── main.rs
│   │    ├── cli
│   │    ├── utils
│   │    └── tests  
│   └── Cargo.toml
│
├── frontend/       # CLI and/or GUI interface
│   ├── personal-git/src/
│   └── TBD
│
├── docs/           # Design documentation and diagrams
├── tests/          # Integration and unit tests
└── README.md
```
---

# Results  

-  
-
- 

---

# Setup  

## Clone the repository  

``` git clone https://github.com/arnaumunozbarrera/Git-Implementation-Rust.git ```

``` cd Git-Implementation-Rust ```

## Install dependencies  

``` cargo clean ``` # Remove previous dependencies  

``` cargo build ``` 

If `Cargo.toml` is not included, manually install:

``` flate2 = "1.0" ```
``` sha1 = "0.10.6" ```
``` sha2 = "0.10" ```
``` ignore = "0.4" ```

## Run the project as a CLI program 

```
cd .\backend\personal-git\
```
### Initialize `.voor` 
```
cargo run -- init
```

This command creates a `.voor` folder containing the main folders and files for later development

### Cat-file
```
cargo run cat-file -p {hash-value}
```

This command displays the content of a created blob file inside the `.voor/objects` folder. {hash-value} refers to the name of the file selected to display its content.

### Hash-object
```
cargo run hash-object -w {file or file_path}
```

This command creates a blob object inside the `.voor/objects` structure to maintain architecture and control of it, the content inside the file is hashed. {file} or {file_path} refers to the name or the file path to create the blob object.

### Diff (Only by hash right now)
```
cargo run diff {hash1} {hash2}
```

This command displays the differences/changes between two files referenced by their hashes.

### Status
```
cargo run status
```

This command displays the file tracking procedure of the file repository indicated. This command allows ignoring references by using a custom made ignore file that is created within the initialization `.voorignore`.

### Add
```
cargo run add {file_path}

or

cargo run add . (add all files within the workspace)
```

This command tracks the current status of the files to the `index` file in assist to other commands such as `status`, `commit`...

### Commit
```
cargo run commit -m {message}
```

This command creates a commit of the current state of the staged / modified files to be later included on the `push` feature for remote repos

### Status
```
cargo run status
```

This command displays the actual status of the repository: modified, staged, deleted & commit files. This command also displays the current development branch at the top by cli.

### Branch
```
cargo run branch # Display current branches available

or

cargo run branch <branch_name> # Create branch

or

cargo run branch -D <branch_name> # Delete branch 
```

### Checkout
```
cargo run checkout <branch_name> # Swap to branch

or

cargo run checkout -b <branch_name> # Create branch & auto-checkout 
```

### Auth for remote commands
```
cargo run -- login <clerk_jwt>   # Store the current Clerk bearer token locally

or

cargo run -- logout              # Remove the stored token from .voor/config
```

## Push/Pull
 
- `remote <url>` stores the remote URL in `.voor/config`.
- `login <clerk_jwt>` stores the bearer token in `.voor/config` so local CLI commands can authenticate against the API.
- `POST /repos/init` creates the repository row in the remote database before the first database-backed sync.
- `init-remote [branch]` initializes the remote database entry for the current local repo, stores `repo_id` in `.voor/config`, and syncs existing local objects if the branch already has commits.
- `push [branch]` collects the branch head commit plus all related commits, trees, and blobs, compresses them, base64-encodes them, and sends them to `/push`.
- `push` now also parses incoming objects on the server and upserts `blobs`, `trees`, `tree_entries`, `commits`, `commits_metadata`, `branches`, and `repo_access_logs`.
- `pull [branch]` requests the latest branch head plus all related objects from `/pull`, saves them locally, updates refs, and restores the working directory if the pulled branch is checked out.
- `sync-db [branch]` re-scans the local branch head and replays missing database state into the remote without requiring a new commit.
- The CLI now prints the server-side database sync result for both `push` and `pull`.
- The server expects `Authorization: Bearer <clerk_jwt>` on all protected routes and uses non-fatal database logging during `push` and `pull`.

## Remote Repository Initialization API

Use this endpoint once per repository before expecting `repo_access_logs` inserts to succeed.

### Endpoint

```text
POST /repos/init
```

### Request body

```json
{
  "repo_id": "personal-git",
  "name": "personal-git",
  "owner_id": "self",
  "default_branch": "master",
  "is_private": false,
  "description": "Personal Git implementation in Rust",
  "readme_path": "README.md",
  "tags": ["rust", "git"],
  "theme": null
}
```

Required fields:

- `repo_id`
- `name`
- `default_branch`
- `is_private`

Notes:

- `repo_id` should match the local folder name because sync derives the repository id from the current working directory.
- `owner_id` is ignored by the backend once the bearer token is validated; the authenticated Clerk user becomes the repository owner in Supabase.
- The endpoint creates a row in `public.repositories` and creates the default branch in `public.branches`.
- The current CLI command `init-remote` calls this endpoint for you and then runs `sync-db` automatically if the branch already has commits.

### Example request

PowerShell:

```powershell
$body = @{
  repo_id = "personal-git"
  name = "personal-git"
  owner_id = "self"
  default_branch = "master"
  is_private = $false
  description = "Personal Git implementation in Rust"
  readme_path = "README.md"
  tags = @("rust", "git")
  theme = $null
} | ConvertTo-Json

Invoke-RestMethod `
  -Method Post `
  -Uri "http://localhost:3000/repos/init" `
  -Headers @{ Authorization = "Bearer $env:CLERK_JWT" } `
  -ContentType "application/json" `
  -Body $body
```

Success response:

```json
{
  "message": "Initialized remote repository 'personal-git'",
  "repo_id": "personal-git",
  "database_action": "Created repository 'personal-git' with default branch 'master'"
}
```

### Error management

- `400 Bad Request`: one or more required fields are missing.
- `401 Unauthorized`: missing or invalid Clerk bearer token.
- `409 Conflict`: the repository already exists in `public.repositories`.
- `503 Service Unavailable`: `SUPABASE_URL` is not configured or the server started without a DB client.
- `500 Internal Server Error`: unexpected database failure while validating or inserting the repository.

### Logs

Server logs:

- Success: `[INFO] Initializing remote repository 'personal-git' for authenticated user '...'`
- Success: `[INFO] Initialized remote repository 'personal-git'`
- Failure: `[WARN] [ERROR] Repository 'personal-git' already exists`

Client sync logs after initialization:

- `push` and `pull` still succeed even if database logging cannot be written.
- `push` stores each incoming object in the filesystem object store and then mirrors supported object types into the database:
  blobs: `hash`, binary `content`, `size`
  trees: `hash`
  tree entries: `tree_hash`, `name`, `type`, `mode`, `hash`
  commits: `hash`, `tree_hash`, `parent_hash`, authenticated Clerk `sub`, commit `message`
  commit metadata: `repo_id`, `commit_hash`, authenticated Clerk `sub`, `message`, calculated `additions`, `deletions`
  branches: update existing `last_commit_hash` or create the branch if missing
- When the DB is empty or partially seeded, the CLI now prints a descriptive skip reason such as:
  `Skipped database log: repository 'personal-git' not found in database`
  or
  `Skipped database log: user '...' not found in database`

## Push/Pull Test Plan

Use three folders with the same repository name for the full test:

- `remote repo`: runs `cargo run -- serve`
- `local repo A`: creates commits and pushes them
- `local repo B`: pulls and verifies them

Important:

- `repo_id` is derived from the current folder name, so all three folders must end with the same directory name.
- The database log is only written when both `SUPABASE_URL` and Clerk auth are configured for the server process.
- `repo_access_logs` also requires matching rows in `public.users` and `public.repositories`.

### 1. Prepare the remote repository folder

Create a folder that will act as the remote repository and move into `backend/personal-git`:

```powershell
cargo run -- init
```

Expected result:

- `.voor` is created in the remote repository.

### 2. Configure database logging and auth for the server

Set the variables before starting the server. Put them in `backend/personal-git/.env` or export them in the terminal:

```powershell
$env:SUPABASE_URL = "<your_supabase_postgres_url>"
$env:CLERK_JWT_ISSUER = "https://<your-clerk-domain>"
$env:CLERK_JWKS_URL = "https://<your-clerk-domain>/.well-known/jwks.json"
$env:CLERK_JWT_AUDIENCE = "<optional_audience>"
$env:PORT = "3000"
```

Expected result:

- `SUPABASE_URL` allows the server to connect to Supabase.
- Clerk auth is enabled for every protected API route.

### 3. Authenticate the user through Clerk

Sign in from the frontend and obtain a Clerk JWT for the current session.

Expected result:

- The JWT contains a valid `sub`.
- The backend can upsert the authenticated user into `public.users` on the first protected request.

### 4. Start the remote server

```powershell
cargo run -- serve
```

Default server URL:

```text
http://localhost:3000
```

Expected result:

- The terminal prints `[INFO] Database connection OK` when Supabase is reachable.
- The terminal prints `[INFO] Clerk auth configured` when the JWT configuration is valid.
- The terminal prints `[INFO] Server running on 127.0.0.1:3000`.

### 5. Prepare local repository A

In the first local working repository:

```powershell
cargo run -- init
cargo run -- remote http://localhost:3000
cargo run -- login <clerk_jwt>
```

Check that `.voor/config` contains:

```ini
[remote "origin"]
url = http://localhost:3000
auth_token = <clerk_jwt>
```

Expected result:

- Local repo A is initialized.
- The remote URL is stored in `.voor/config`.

### 6. Initialize the repository in the remote database

Run the CLI bootstrap command from local repository A:

```powershell
cargo run -- init-remote
```

Expected result:

- A row is created in `public.repositories`.
- A default `master` branch is created in `public.branches`.
- `.voor/config` stores the generated mapping:

```ini
[remote "origin"]
url = http://localhost:3000
repo_id = personal-git
auth_token = <clerk_jwt>
```

- If the branch already has commits, the command also pushes object state into the DB using `sync-db`.

### 7. Create data in local repository A

```powershell
Set-Content hello.txt "hello from local A"
cargo run -- add hello.txt
cargo run -- commit -m "first sync commit"
```

Expected result:

- A new blob, tree, and commit are created under `.voor/objects`.
- `.voor/refs/heads/master` points to the new commit hash.

### 8. Push from local repository A

```powershell
cargo run -- push
```

Expected result:

- The current branch head is read from `.voor/refs/heads/master`.
- The latest commit and all related objects are sent to `/push`.
- The server parses incoming blob/tree/commit objects and mirrors them into the database before updating branch state and writing the push log.
- The CLI prints `Pushed branch 'master' at <hash>`.
- The CLI prints `Sent <n> objects`.
- The CLI prints one database status line:
  `Synced <x> blobs, <y> trees, <z> commits into database; ...`
  or
  `Skipped database log: ...`

### 9. Verify push in the database

Run this query in Supabase SQL Editor or any PostgreSQL client connected to the same database:

```sql
select action, repo_id, user_id, metadata, created_at
from repo_access_logs
where action = 'push'
order by created_at desc
limit 5;
```

Expected result:

- A new `push` row exists for the repository.
- `metadata` contains the branch name, head commit hash, and object count.

### 10. Prepare local repository B

In a second local repository:

```powershell
cargo run -- init
cargo run -- remote http://localhost:3000
cargo run -- login <clerk_jwt>
```

Expected result:

- Local repo B is initialized and points to the same remote.

### 11. Pull into local repository B

```powershell
cargo run -- pull master
```

Expected result:

- Objects are downloaded from `/pull`.
- Received objects are written into `.voor/objects`.
- `.voor/refs/heads/master` is updated to the pulled commit.
- If `master` is the checked out branch, the working directory is restored from the pulled tree.
- The CLI prints `Received <n> objects`.
- The CLI prints one database status line:
  `Logged pull action into repo_access_logs`
  or
  `Skipped database log: ...`

### 12. Verify pulled content in local repository B

```powershell
Get-Content hello.txt
cargo run -- status
```

Expected result:

- `hello.txt` exists with the content from local repository A.
- `status` shows no pending changes if nothing was edited after pull.

### 13. Verify pull in the database

```sql
select action, repo_id, user_id, metadata, created_at
from repo_access_logs
where action = 'pull'
order by created_at desc
limit 5;
```

Expected result:

- A new `pull` row exists for the repository.
- `metadata` contains the branch name, head commit hash, and object count.

### 14. Test branch-specific push/pull

In local repository A:

```powershell
cargo run -- checkout -b feature-sync
Set-Content feature.txt "branch content"
cargo run -- add feature.txt
cargo run -- commit -m "feature commit"
cargo run -- push feature-sync
```

In local repository B:

```powershell
cargo run -- branch feature-sync
cargo run -- checkout feature-sync
cargo run -- pull feature-sync
```

Expected result:

- The remote stores and serves branch-specific heads.
- Pull updates `refs/heads/feature-sync`.
- Checkout/pull restores `feature.txt` into the working tree.
- The database records one `push` and one `pull` for `feature-sync`.

## Related Checks

### Verify object storage layout

After `add` or `commit`, inspect:

```powershell
Get-ChildItem .voor\objects -Recurse
```

You should see objects stored by hash prefix, for example:

```text
.voor/objects/ab/cdef...
```

### Verify commit and tree objects exist

```powershell
Get-Content .voor\refs\heads\master
cargo run -- cat-file -p <blob_hash>
```

Notes:

- `cat-file -p` currently prints blob contents.
- Trees and commits are stored with correct headers internally even though there is no dedicated pretty-printer yet.

### Missing branch error

```powershell
cargo run -- pull branch-that-does-not-exist
```

Expected result:

- The command returns an error for the missing branch.

### Missing object error

If the remote branch points to an object that is not present on the server, `/pull` should fail with a missing object error.

## Notes

- The sync flow currently identifies the repository by the working directory name on client and server.
- The server keeps exact objects in its own `.voor/objects` store so commit hashes remain stable during push/pull.
- `/repos`, `/users`, `/repos/init`, `/push`, `/pull`, and `/sync-db` are protected by Clerk bearer-token auth.
- `/sync-db` also depends on `SUPABASE_URL` because it validates and writes repository state into PostgreSQL.
- `repo_access_logs` is best-effort. Failed inserts no longer turn a successful `push` or `pull` into an HTTP error.
- `push`, `pull`, `init-remote`, and `sync-db` now authenticate with the bearer token stored in `.voor/config` after `cargo run -- login <clerk_jwt>`.
- `push`, `pull`, and `sync-db` now expose the database sync or logging result directly in the CLI output.

## Concurrency & Locking

This project now uses a repository-scoped lock and atomic file replacement for critical local mutations.

### Brief explanation

- A single lock file is created at `.voor/locks/repo.lock` before mutating repository state.
- The lock is used by the main write paths: `init`, `add`, `commit`, branch create/delete, checkout, `init-remote`, `pull`, `push` snapshot creation, `sync-db`, and the server-side `/push`, `/pull`, `/sync-db` handlers.
- When the lock is held, other mutating operations wait for it instead of racing on `HEAD`, refs, index state, config, or object files.
- Lock files older than 5 minutes are treated as stale and removed automatically.
- Critical files are now written through a temporary file and then moved into place:
  `.voor/HEAD`
  `.voor/index`
  `.voor/config`
  `.voor/refs/heads/*`
  `.voor/objects/*`

### Why this was added

- Prevent concurrent commands from overwriting `HEAD` or branch refs.
- Prevent partial writes to the index, config, and object store when two processes write at once.
- Keep `pull` / checkout-style working tree restoration from interleaving with local commit or branch operations.
- Give the API and CLI the same single-writer behavior on the local repository storage.

### Operational notes

- Lock wait timeout: 15 seconds.
- Poll interval while waiting: 100 ms.
- Stale lock TTL: 5 minutes.
- The current model is intentionally conservative: one writer at a time per repository.
- The database layer still uses database constraints and upserts, but the local repository filesystem is now protected by the repo lock.

## Auth

This project now uses **Clerk** for authentication and **Supabase Postgres** for persistence.

The API accepts Clerk bearer tokens on every protected route. After the token is validated against the configured JWKS, the backend uses the Clerk `sub` as the user id in Supabase, updates `public.users` when needed, and ignores any `user_id` sent in the request body. For the current scope there is no collaborator model: anyone authenticated and working with a local copy of the repository is treated as the acting owner for their operations.

The backend reads `CLERK_JWT_ISSUER` and `CLERK_JWKS_URL` from the environment. `CLERK_JWT_AUDIENCE` is optional and only needed if your Clerk template includes an audience claim that you want to enforce.

In Clerk, the token template used by the project is `jwt-token`. The backend does not read the template name directly, but the frontend or any local CLI flow that fetches a token must request that template. The token should contain the standard claims `sub`, `iss`, and `exp`. It can also include `email` and `username`; those two fields are used to keep the `users` table populated with readable account data.

If you want the template to match the current implementation, this payload is enough:

```json
{
  "email": "{{user.primary_email_address.email_address}}",
  "username": "{{user.username}}"
}
```

The local CLI uses the same token model as the frontend. Once you have a valid token generated from the `jwt-token` template, store it in the repository config with:

```powershell
cargo run -- login <clerk_jwt>
```

This writes the token to `.voor/config` as `auth_token`. To clear it again:

```powershell
cargo run -- logout
```

The simplest deployment remains the current one: a single backend process, Supabase as the database, and Clerk issuing the bearer token used by both the frontend and the local CLI. No additional auth service or collaborator table is required for this version.
