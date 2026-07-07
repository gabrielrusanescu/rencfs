# Python Binding - Step 2: Defining and Exporting Data Structures

In this step, we focus on identifying and exposing (wrapping) the necessary data structures and enums from Rust to Python, using PyO3 macros (`#[pyclass]`, `#[pymethods]`). 

According to the plan, we need to bind the internal types used by `EncryptedFs` to Python.

## 1. Mapping Basic Enums

Let's take `FileType` and `Cipher` as examples. These must be exposed in Python so that the user can correctly set the arguments when calling functions like `create`.

### Conceptual Code (`rencfs-python/src/types.rs`):
```rust
use pyo3::prelude::*;
use rencfs::{FileType as RustFileType, Cipher as RustCipher};

/// Wrapper for file type
#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum FileType {
    RegularFile,
    Directory,
}

impl From<RustFileType> for FileType {
    fn from(kind: RustFileType) -> Self {
        match kind {
            RustFileType::RegularFile => FileType::RegularFile,
            RustFileType::Directory => FileType::Directory,
        }
    }
}

impl From<FileType> for RustFileType {
    fn from(kind: FileType) -> Self {
        match kind {
            FileType::RegularFile => RustFileType::RegularFile,
            FileType::Directory => RustFileType::Directory,
        }
    }
}

/// Wrapper for supported ciphers
#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum Cipher {
    Aes256Gcm,
    ChaCha20Poly1305,
}

// Similar `From` implementations for bidirectional conversion.
```

## 2. Mapping Attribute Structures (`CreateFileAttr`, `FileAttr`)

When creating or reading a file, we obtain attributes. PyO3 allows defining `#[new]` methods (constructor) and properties with `#[pyo3(get, set)]`.

```rust
use rencfs::CreateFileAttr as RustCreateFileAttr;

#[pyclass]
#[derive(Clone)]
pub struct CreateFileAttr {
    #[pyo3(get, set)]
    pub kind: FileType,
    #[pyo3(get, set)]
    pub perm: u16,
    #[pyo3(get, set)]
    pub uid: u32,
    #[pyo3(get, set)]
    pub gid: u32,
}

#[pymethods]
impl CreateFileAttr {
    #[new]
    fn new(kind: FileType, perm: u16, uid: u32, gid: u32) -> Self {
        Self { kind, perm, uid, gid }
    }
}

// Function to convert the Python class to the Rust structure:
impl From<CreateFileAttr> for RustCreateFileAttr {
    fn from(attr: CreateFileAttr) -> Self {
        RustCreateFileAttr {
            kind: attr.kind.into(),
            perm: attr.perm,
            uid: attr.uid,
            gid: attr.gid,
            flags: 0, // default values required in Rust
        }
    }
}
```

## 3. Safe Handling of Sensitive Data (`SecretString`, `SecretVec`)

A crucial aspect (#7 of the objectives): `rencfs` uses memory zeroization via `SecretString` (from libraries like `secrecy` or `shush-rs`).
When Python sends the password to Rust, it sends it as a standard string. In Rust, we must immediately convert it to a secure type that will "erase" (zeroize) the memory when the variable goes out of scope.

```rust
// Example: When accepting a password from a Python argument, we will immediately pack it:
use rencfs::SecretString;

// In the instantiation function, we will take a `String` from Python:
// fn init_fs(password: String) {
//     let secret_password = SecretString::new(Box::new(password.into()));
//     // The internal Rust memory is now protected.
// }
```
*Note: Python does not guarantee memory erasure for its own strings when garbage collected, but at least the Rust part will be safe and use protected internal memory for the rest of the instance's lifecycle.*

## 4. Mapping Errors (`FsError`)

Rust exceptions must be mapped to Python exceptions (e.g., `ValueError`, `FileNotFoundError`).

```rust
use pyo3::exceptions::{PyFileNotFoundError, PyValueError, PyRuntimeError};
use rencfs::FsError;

pub struct PyFsError(FsError);

impl From<FsError> for PyFsError {
    fn from(err: FsError) -> Self {
        PyFsError(err)
    }
}

impl From<PyFsError> for PyErr {
    fn from(err: PyFsError) -> PyErr {
        match err.0 {
            FsError::InodeNotFound => PyFileNotFoundError::new_err("Inode not found"),
            FsError::InvalidInput(msg) => PyValueError::new_err(msg.to_string()),
            // The rest of the errors mapped to PyRuntimeError
            e => PyRuntimeError::new_err(e.to_string()),
        }
    }
}
```

## Why did we do this?
We could have tried to attach `#[pyclass]` directly to the structures in the `rencfs` crate, but this would have invaded the main source code with Python-specific annotations (and would have required adding the `pyo3` dependency to the main logic body). Through the approach with wrappers (`impl From<RustType> for PyType`), we decouple the core library 100% from its interface for Python.

**Status:** The data part is defined in principle. We can move to Step 3 (Implementation of methods for the main class `EncryptedFs`).
