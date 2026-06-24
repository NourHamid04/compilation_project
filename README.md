# Compilation Techniques Project

Implementation of MiniImp and MiniFun developed for the Compilation Techniques course.



---

# Overview

This project implements a complete compiler/interpreter infrastructure for two educational programming languages:

- **MiniImp** (Imperative Language)
- **MiniFun** (Functional Language)

The project was developed incrementally through multiple fragments and includes:

---

# Implemented Fragments

## Fragment 1 – MiniImp Interpreter
## Fragment 2 – MiniFun Interpreter
## Fragment 3 – MiniFun Type Checker
## Fragment 5 – Control Flow Graph
## Fragment 6 – Dataflow Analysis
## Fragment 7 – Compiler Optimizations
## Fragment 8 – LLVM Backend

# Technology Stack

| Component | Technology |
|------------|------------|
| Language | Rust |
| Build System | Cargo |
| Parsing | Recursive-Descent Parser |
| Backend | LLVM IR |
| Optimization | LLVM mem2reg |
| Object Generation | llc |
| Linking | clang |

---

# Requirements

Install:

- Rust (latest stable version)
- Cargo
- LLVM
- Clang

Verify installation:

```bash
rustc --version
cargo --version
clang --version
opt --version
llc --version
```


# Building the Project

Clone the repository:

```bash
git clone https://github.com/NourHamid04/compilation_project.git
cd compilation_project
```

Build:

```bash
cargo build
```

Build in release mode:

```bash
cargo build --release
```

---

# Running the Project

Run:

```bash
cargo run
```

Run optimized version:

```bash
cargo run --release
```

---

# LLVM Compilation Pipeline

Generate LLVM IR:

```bash
cargo run
```

Apply SSA optimization:

```bash
opt -passes="mem2reg" program.ll -S -o program_opt.ll
```

Generate object file:

```bash
llc -filetype=obj program_opt.ll -o program.o
```

Link executable:

```bash
clang wrapper.c program.o -o program
```

Run executable:

```bash
./program
```

---

# Compilation Workflow

```text
Source Program
      ↓
Lexer
      ↓
Parser
      ↓
AST
      ↓
Type Checker
      ↓
Interpreter
      ↓
CFG Generation
      ↓
Dataflow Analysis
      ↓
Optimizations
      ↓
LLVM IR
      ↓
mem2reg
      ↓
Object Code
      ↓
Executable
```

---
