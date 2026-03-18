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
-  OB1 - **Develop a functional system:** a software system that enables the execution of version-control operations in collaborative environments, both in local and remote workspaces.
-  OB3 **Develop an analytical interface:** design and build a system focused on analytically visualising repository status through dashboards and visual elements, enabling an intuitive understanding between the user and the system.
- OB4 **Provide relevant metrics:** provide the user with relevant metrics and indicative, dynamic visualisations that improve both capability and ease of decision-making regarding project evolution and repository activity.
- OB5 **Explore the viability of an alternative solution:** create a proprietary implementation as an alternative to traditional version-control systems, from a modern technical and architectural perspective, for a more user-adapted experience.

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
│   ├── src/
│   │    ├── main.rs
│   │    ├── cli
│   │    ├── utils
│   │    └── tests  
│   └── Cargo.toml
│
├── frontend/       # CLI and/or GUI interface
│   ├── src/
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