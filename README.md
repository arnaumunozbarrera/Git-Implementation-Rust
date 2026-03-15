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
-  
-  
-  

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

-  
-  
-  

---

## Tech Stack

- **Language:** Rust  
-  
-  

---

## Project Structure

```
/
├── backend/        # Core VCS engine
│   ├── src/
│   └── Cargo.toml
│
├── frontend/       # CLI and/or GUI interface
│   ├── src/
│   └── Cargo.toml
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

```
git clone https://github.com/arnaumunozbarrera/Git-Implementation-Rust.git
cd Git-Implementation-Rust
```

## Install dependencies  

```
```

If `Cargo.toml` is not included, manually install:

```
```

## Run the project  

```
cd .\personal-git\
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