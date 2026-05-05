# Voor Executable Validation

This document provides a brief step-by-step plan to execute and validate the downloadable `voor` executable.

---

## Goal

Validate that the installed executable can be called directly from a terminal:

```powershell
voor init
```

with no `cargo run` and no absolute path.

---

## Step-by-step Execution Plan

1. Build and publish the release assets through GitHub Actions.
2. Download the correct ZIP release asset for the target platform.
3. Extract the executable and place it in a directory on `PATH`.
4. Open a fresh terminal session.
5. Validate the executable metadata.
6. Validate local repository commands.
7. Validate nested-folder repository discovery.
8. Validate auth and remote configuration.
9. Validate `push`, `pull`, and `serve`.

---

## Validation Steps

### 1. Validate install visibility

Windows:

```powershell
voor --version
where.exe voor
```

Linux / macOS:

```bash
voor --version
which voor
```

Expected result:

- the shell resolves `voor`
- the version is printed successfully

### 2. Validate help output

```powershell
voor --help
```

Expected result:

- subcommands are listed
- `serve`, `login`, `push`, and `pull` are present

### 3. Validate local repository commands

```powershell
mkdir demo-repo
cd demo-repo
voor init
Set-Content hello.txt "hello"
voor add hello.txt
voor commit -m "first commit"
voor status
```

Expected result:

- `.voor` is created
- the file is staged and committed
- `status` reports no pending changes after the commit

### 4. Validate nested-folder repository discovery

```powershell
mkdir src
Set-Location src
voor status
```

Expected result:

- `voor` finds the parent `.voor` directory automatically

### 5. Validate auth commands

```powershell
voor login <clerk_jwt>
voor logout
```

Expected result:

- login stores the token in the platform user config
- logout removes it cleanly

### 6. Validate remote configuration

```powershell
voor remote http://localhost:3000
voor init-remote
```

Expected result:

- `.voor/config` stores the remote URL
- remote initialization succeeds when the server is available and auth is valid

### 7. Validate push and pull

```powershell
voor push
voor pull master
```

Expected result:

- push uploads objects and prints sync status
- pull restores objects and updates refs and the working tree when appropriate

### 8. Validate server mode

```powershell
voor serve
```

Expected result:

- the HTTP server starts from the installed executable
- no source checkout path is required at runtime

---

## Minimum Release Gate

Do not consider the executable distribution ready until all of these pass:

- `voor --version`
- `voor --help`
- `voor init`
- `voor add`
- `voor commit`
- `voor login`
- `voor push`
- `voor pull`
- `voor serve`
