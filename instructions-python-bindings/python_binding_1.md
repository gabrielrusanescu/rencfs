# Python Binding - Step 1: Setup and Initializing PyO3

According to the established plan for adding Python bindings to the `rencfs` project (reference: Issue #203), in this first step we will focus on configuring the environment, setting up PyO3, and creating the basic structure to expose the library to Python.

*Note: According to your instructions, we have currently NOT modified any source or configuration files. This is just the documentation of the steps we are about to execute.*

## 1. Configuring the Workspace and Creating the `rencfs-python` Package

To maintain a clean architecture and avoid mixing `cdylib` (required for Python) with the main Rust library/binary, we will create a dedicated sub-package (crate) named `rencfs-python`.

### Exact Steps:
1. We will modify the `Cargo.toml` in the project root to transform it into a workspace:
   ```toml
   [workspace]
   members = [
       ".",
       "rencfs-python"
   ]
   ```
2. We will run the command `cargo new --lib rencfs-python` to generate the new crate.

## 2. Configuring Dependencies for Python (PyO3)

In the new `rencfs-python` folder, we will edit `Cargo.toml` to include PyO3 and specify the crate type as a C-compatible dynamic library (`cdylib`), which Python can load.

### What we add to `rencfs-python/Cargo.toml`:
```toml
[package]
name = "rencfs-python"
version = "0.1.0"
edition = "2021"

[lib]
name = "rencfs"
# "cdylib" is necessary to produce a shared library (.so/.pyd) that Python can import.
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.20.0", features = ["extension-module", "anyhow"] }
# We will also add pyo3-asyncio later for async methods (tokio)
# Dependency towards our main library
rencfs = { path = "../" }
```

## 3. Writing the Boilerplate for the Python Module

In `rencfs-python/src/lib.rs`, we will define the entry point for the Python extension. This code will initialize the `rencfs` module when imported in Python with `import rencfs`.

### Initial Code (`rencfs-python/src/lib.rs`):
```rust
use pyo3::prelude::*;

/// An example of a simple function to test the integration.
#[pyfunction]
fn version() -> PyResult<String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// Entry point for the Python module. The function name must match `name` from `[lib]`.
#[pymodule]
fn rencfs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    // Here we will later add the EncryptedFs class and other structures (FileAttr, FsError etc.)
    Ok(())
}
```

## 4. Configuring Maturin

To build and test the Python package, we will use `maturin`, a dedicated build system for PyO3.
1. Create a virtual environment: `python -m venv .venv`
2. Activate the environment: `source .venv/bin/activate` (on Linux/macOS) or `.venv\Scripts\activate` (on Windows).
3. Install Maturin: `pip install maturin`
4. To build the module and install it directly into the virtual environment for testing: `maturin develop`

## Why did we choose this approach?
- **Separation of concerns:** Creating a separate `rencfs-python` crate avoids polluting the main `Cargo.toml` with Python-specific settings (`cdylib`, heavy dependency on `pyo3`).
- **Compatibility:** PyO3 is the de facto standard for Rust-Python bindings currently, being much safer and more ergonomic than SWIG (which you mentioned as an alternative, but PyO3 is preferred here).
- **Easy testability:** With `maturin develop`, we will be able to iterate quickly by writing Rust code and testing it directly from the console with Python scripts in the virtual environment.

**Status:** Awaiting your confirmation for this plan/documentation and eventually proceeding to Step 2 (Defining and exporting data structures such as `EncryptedFs`, `FsError`, etc.). We have not modified any source files.
