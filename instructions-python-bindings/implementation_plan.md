# Implementation Plan: Python Bindings for `rencfs`

This plan outlines the steps to implement Python bindings for the `rencfs` project using `PyO3`, according to Issue #203.

## Goal Description
Create a Python library (`rencfs-python`) that exposes the core `EncryptedFs` methods from Rust, allowing Python developers to programmatically create, manage, and access encrypted file systems natively without relying on FUSE mounting.

## Proposed Changes

### 1. Workspace Configuration
To keep the main `rencfs` crate clean, we will convert the project into a Cargo workspace.
#### [MODIFY] [Cargo.toml](file:///home/admin1/rosedu/rencfs/rencfs/Cargo.toml)
- Append a `[workspace]` section to include `.` and a new `python` directory.

### 2. Python Bindings Crate
We will create a new crate specifically for the Python bindings.
#### [NEW] [python/Cargo.toml](file:///home/admin1/rosedu/rencfs/rencfs/python/Cargo.toml)
- Define the `rencfs-python` crate with `crate-type = ["cdylib"]`.
- Add dependencies: `pyo3`, `rencfs` (path dependency), and `tokio`.

#### [NEW] [python/pyproject.toml](file:///home/admin1/rosedu/rencfs/rencfs/python/pyproject.toml)
- Define the build system for Python using `maturin`.

### 3. Core Bindings Implementation
We will implement the wrappers in the `python/src` directory.

#### [NEW] [python/src/lib.rs](file:///home/admin1/rosedu/rencfs/rencfs/python/src/lib.rs)
- Define the `#[pymodule]` named `rencfs_python`.
- Add bindings for basic data structures: `Cipher`, `CreateFileAttr`, `FileType`.
- Add a Python class wrapper `PyEncryptedFs` around `Arc<EncryptedFs>`.
- Implement `#[pymethods]` for `PyEncryptedFs` to expose:
  - `new` (Constructor)
  - `create`
  - `open`
  - `read`
  - `write`
  - `flush`
  - `release`
  - `read_dir`
  - `passwd`
  - *Note: Since `EncryptedFs` methods are async, we will use a dedicated `tokio` runtime inside the wrapper to expose them as synchronous Python methods for ease of use, or explore `pyo3-asyncio`.*

### 4. Step-by-Step Documentation
Following the requested workflow, I will create documentation files (e.g., `python binding 1.md`, `python binding 2.md`) explaining each step of the implementation as I do it.

## User Review Required
> [!IMPORTANT]
> - Is it acceptable to convert the root `Cargo.toml` into a workspace? This is the standard way to add secondary crates like bindings.
> - Do you prefer the Python API to be synchronous (blocking) or asynchronous (`async def`)? For a filesystem library, synchronous is often easier to use in Python scripts, but I can do async if preferred. I will default to synchronous wrappers internally executing tokio futures.

## Verification Plan

### Automated Tests
- Write a Python test script (`python/tests/test_basic.py`) using `pytest`.
- The tests will initialize `PyEncryptedFs`, create a file, write data, read it back, and assert the contents.

### Manual Verification
- Build the wheel locally using `maturin develop`.
- Open a Python REPL and test the bindings interactively.
