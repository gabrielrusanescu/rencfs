# Python Binding - Step 3: Implementing the Core Classes (EncryptedFs)

Now that we have defined and mapped the data types between Rust and Python, we will implement the main "engine": the `EncryptedFs` class. This is the critical step, as it is where we expose the actual functional API.

## 1. Creating the Wrapper for `EncryptedFs`

In Rust, `EncryptedFs` is created asynchronously and returns an `Arc<EncryptedFs>`. From Python, we want a smooth instantiation. Because our Rust functions use `tokio`, we will run an internal (blocking) executor for the Python calls, so that the API exposed in Python is synchronous (easier to use for most users). If we later want a fully async API in Python, we will use the `pyo3-asyncio` package. 

For now, we will expose a synchronous Python API that, internally in Rust, runs asynchronously via `block_on`.

### Conceptual Code (`rencfs-python/src/fs.rs`):

```rust
use pyo3::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Using the types created in the previous step
use crate::types::{Cipher, CreateFileAttr, FileAttr, PyFsError}; 
use rencfs::{EncryptedFs as RustEncryptedFs, PasswordProvider, SecretString};

// A simple implementer to deliver the password
struct PyPasswordProvider {
    password: SecretString,
}

impl PasswordProvider for PyPasswordProvider {
    fn get_password(&self) -> Option<SecretString> {
        Some(self.password.clone())
    }
}

#[pyclass]
pub struct EncryptedFs {
    // Keep the internal Rust instance
    inner: Arc<RustEncryptedFs>,
    // Keep a Tokio runtime instance dedicated to the Python interface
    rt: Runtime,
}

#[pymethods]
impl EncryptedFs {
    /// Constructor for the class in Python
    /// Equivalent to: fs = EncryptedFs("/data", "my_secret_pass", Cipher.Aes256Gcm, False)
    #[new]
    fn new(
        data_dir: String,
        password: String,
        cipher: Cipher,
        read_only: bool,
    ) -> PyResult<Self> {
        let rt = Runtime::new()?;
        
        let provider = Box::new(PyPasswordProvider {
            password: SecretString::new(Box::new(password.into())),
        });

        // Run the asynchronous initialization code
        let inner_result = rt.block_on(async {
            RustEncryptedFs::new(
                PathBuf::from(data_dir),
                provider,
                cipher.into(),
                read_only,
            )
            .await
        });

        match inner_result {
            Ok(inner) => Ok(EncryptedFs { inner, rt }),
            Err(e) => Err(PyFsError::from(e).into()),
        }
    }
}
```

## 2. Implementing File Operations

Adding the CRUD methods over the file system (create, exists, is_file, etc.).

```rust
#[pymethods]
impl EncryptedFs {
    /// Check if an inode exists
    fn exists(&self, ino: u64) -> bool {
        self.inner.exists(ino)
    }

    /// Create a new file/directory.
    fn create(
        &self,
        parent_ino: u64,
        name: String,
        attr: CreateFileAttr,
        read: bool,
        write: bool,
    ) -> PyResult<(u64, FileAttr)> {
        let secret_name = SecretString::new(Box::new(name.into()));
        let rust_attr: rencfs::CreateFileAttr = attr.into();

        let result = self.rt.block_on(async {
            self.inner.create(parent_ino, &secret_name, rust_attr, read, write).await
        });

        match result {
            Ok((ino, returned_attr)) => Ok((ino, returned_attr.into())), // We need impl From<RustFileAttr> for FileAttr
            Err(e) => Err(PyFsError::from(e).into()),
        }
    }
    
    // Similarly, will be implemented:
    // fn open(&self, ...) -> PyResult<u64>
    // fn read(&self, ...) -> PyResult<Vec<u8>> // or objects of type buffer (PyBuffer) for performance
    // fn write(&self, ...) -> PyResult<u32>
    // fn remove_file(&self, ...) -> PyResult<()>
}
```

## 3. Performance for I/O Operations (Read/Write)

When working with large blocks of data (e.g., 1MB), copying the data from the vector of `u8` in Rust to a Python structure (`list` or `bytes`) of type `Vec<u8>` brings a small overhead (the data is copied from Rust memory manager to Python garbage collector). 
In PyO3, we can use the macros and methods for "Buffer Protocol" (e.g., `&[u8]` to `PyBytes`), which are much faster. We will analyze this zero-copy transfer specifically when we effectively map `read` and `write`.

## Why did we choose `block_on` (Synchronous API in Python)?
The standard FUSE ecosystem (or the development of scripting in Python) often prefers blocking synchronous functions for basic I/O operations (such as standard calls `os.read`, `os.write`). Building and maintaining an internal `tokio` runtime hides the complexity for the Python programmer: they just want to instantiate the class and call `fs.read()`.

**Status:** With Step 3 completed theoretically, we have everything necessary to open a file and read from it. The next step involves unit testing (via `pytest`) and documentation.
