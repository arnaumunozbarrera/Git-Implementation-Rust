# Voor Download and Installation

This document describes the intended end-user installation flow for the downloadable `voor` executable.

---

## Release Assets

GitHub Releases should publish one ZIP file per supported platform:

- `voor-windows-x86_64.zip`
- `voor-linux-x86_64.zip`
- `voor-macos-x86_64.zip`

Each ZIP contains the single executable:

- Windows: `voor.exe`
- Linux / macOS: `voor`

The current design keeps all CLI functionality in the same executable, including:

- local repository commands
- auth commands
- remote synchronization commands
- `voor serve`

---

## Windows Installation

1. Download `voor-windows-x86_64.zip` from GitHub Releases.
2. Extract `voor.exe`.
3. Move `voor.exe` to a stable folder such as:

```text
%LOCALAPPDATA%\Programs\voor\
```

4. Add that folder to the user `PATH`.
5. Open a new terminal.
6. Validate:

```powershell
voor --version
where.exe voor
```

---

## Linux Installation

1. Download `voor-linux-x86_64.zip`.
2. Extract `voor`.
3. Mark the file executable if needed:

```bash
chmod +x voor
```

4. Move it to a directory on `PATH`, for example:

```bash
mkdir -p ~/.local/bin
mv voor ~/.local/bin/voor
```

5. Open a new terminal.
6. Validate:

```bash
which voor
voor --version
```

---

## macOS Installation

1. Download `voor-macos-x86_64.zip`.
2. Extract `voor`.
3. Mark the file executable if needed:

```bash
chmod +x voor
```

4. Move it to a directory on `PATH`, for example:

```bash
sudo mv voor /usr/local/bin/voor
```

5. Open a new terminal.
6. Validate:

```bash
which voor
voor --version
```

---

## After Installation

Once installed, the intended usage is:

```powershell
voor init
voor add .
voor commit -m "message"
voor login
voor push
voor pull master
voor serve
```

No Cargo command and no absolute executable path should be required.

---

## Troubleshooting

### `voor` is not recognized

Cause:

- the executable directory is not on `PATH`

Check:

```powershell
where.exe voor
```

or

```bash
which voor
```

### Auth token not found

Use:

```powershell
voor login
```

or set:

```powershell
$env:VOOR_AUTH_TOKEN = "<clerk_jwt>"
```

### Not a repository

Initialize the current folder first:

```powershell
voor init
```

### Push or pull fails

Verify:

- `.voor/config` contains `url = http://localhost:3000`
- auth is available through browser login or `VOOR_AUTH_TOKEN`
- the server is reachable
- Clerk and Supabase configuration are valid on the server side
