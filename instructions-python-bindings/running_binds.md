# How to Run and Test the Python Bindings for rencfs

This document provides a step-by-step guide to build, install, and test the Python bindings for the `rencfs` encrypted filesystem.

## Prerequisites

- Rust toolchain (nightly) - as specified in `rust-toolchain.toml`
- Python 3.13 (or compatible) - the bindings are built for CPython 3.13
- `maturin` - the build system for PyO3 projects
- Git (to clone the repository if not already done)

## Step 1: Clone the Repository (if not already done)

```bash
git clone https://github.com/xoriors/rencfs.git
cd rencfs
```

## Step 2: Set Up the Python Virtual Environment

The `rencfs-python` crate includes a pre-configured virtual environment (`.venv`). To activate it:

```bash
# From the root of the repository
source rencfs-python/.venv/bin/activate
```

Alternatively, you can create your own virtual environment:

```bash
python3 -m venv .venv
source .venv/bin/activate
```

Then install `maturin`:

```bash
pip install maturin
```

## Step 3: Build and Install the Bindings

Navigate to the `rencfs-python` directory and build the package in development mode:

```bash
cd rencfs-python
maturin develop
```

This command will compile the Rust extension and install it in the active Python environment.

### Note on Build Issues

During development, we encountered a compilation error related to the `rustix` dependency:

```
error: attributes starting with `rustc` are reserved for use by the `rustc` compiler
```

This is due to a version mismatch between the `rustix` crate in the lock file and the nightly Rust toolchain. To resolve this, you may need to update the `rustix` dependency to a version compatible with your Rust toolchain (e.g., `rustix = "1.1.4"`). You can do this by running:

```bash
cargo update -p rustix --precise 1.1.4
```

Then retry `maturin develop`.

## Step 4: Run the Tests

The Python bindings include a test suite in `rencfs-python/tests/test_basic.py`. To run the tests:

```bash
pytest
```

You should see output similar to:

```
============================= test session starts ==============================
collected 3 items

tests/test_basic.py::test_encryptedfs_init PASSED
tests/test_basic.py::test_encryptedfs_create PASSED
tests/test_basic.py::test_encryptedfs_read_write PASSED

============================== 3 passed in 0.56s ===============================
```

## Step 5: Manual Testing (Optional)

You can also test the bindings interactively in a Python REPL:

```bash
python3
```

Then in the Python interpreter:

```python
import rencfs
from rencfs import EncryptedFs, Cipher, CreateFileAttr, FileType

# Create a temporary directory for testing
import tempfile
import os
import shutil

test_dir = tempfile.mkdtemp()
try:
    # Initialize the encrypted filesystem
    fs = EncryptedFs(test_dir, "my_secure_password", Cipher.Aes256Gcm, False)
    
    # Create a file
    attr = CreateFileAttr(FileType.RegularFile, 0o644, 1000, 1000)
    ino, ret_attr = fs.create(1, "testfile.txt", attr, True, True)
    
    # Write data
    fh = fs.open(ino, False, True)
    data = b"hello encrypted world!"
    written = fs.write(ino, 0, data, fh)
    fs.flush(fh)
    fs.release(fh)
    
    # Read data
    fh2 = fs.open(ino, True, False)
    read_data = fs.read(ino, 0, 100, fh2)
    assert read_data == data
    fs.release(fh2)
    
    print("All manual tests passed!")
finally:
    # Clean up
    shutil.rmtree(test_dir, ignore_errors=True)
```

## Step 6: Running the Full Test Suite (if available)

The project may have additional tests in the `tests/` directory (for the core Rust library). To run the Rust tests (excluding the Python bindings):

```bash
# From the root of the repository
cargo test --release --all-features
```

Note: The Python bindings are not included in the default Rust test suite.

## Summary

- The Python bindings expose the core `EncryptedFs` API to Python.
- The build process uses `maturin` to compile the Rust extension.
- The test suite validates basic functionality (initialization, creation, read/write).
- Ensure the `rustix` dependency is compatible with your Rust toolchain to avoid build errors.

For more information, refer to the original issue and the implementation plan in the `instructions-python-bindings` directory.
