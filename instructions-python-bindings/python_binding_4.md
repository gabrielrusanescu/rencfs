# Python Binding - Step 4: Complete I/O Operations (Read, Write, Open, Flush, Release)

This document summarizes how we integrated the complete I/O operations into the PyO3 API of the `EncryptedFs` class, allowing Python applications to not only create but also read and write actual data (bytes) in encrypted files.

## 1. Opening Files (`open`)

The `open` method allocates an internal file "handle" in Rust and prepares the buffers. This handle is necessary for the rest of the operations.

```rust
fn open(&self, ino: u64, read: bool, write: bool) -> PyResult<u64>
```
**Explanation:** Transmits the asynchronous command `self.inner.open` through `rt.block_on`. Receives an `ino` (inode number) and returns a `handle` (u64).

## 2. Writing Data (`write`)

The `write` method exposes the ability to send `bytes` from Python to the file system, which will be encrypted and placed on disk.

```rust
fn write(&self, ino: u64, offset: u64, buf: &[u8], handle: u64) -> PyResult<usize>
```
**Explanation:**
- `buf: &[u8]` from the Rust signature is automatically mapped by PyO3 to receive objects that support Python Buffer Protocol (e.g., `bytes` or `bytearray`).
- Responds with the number of bytes successfully written.

## 3. Reading Data (`read`)

To read the content of the decrypted file:

```rust
fn read(&self, ino: u64, offset: u64, size: usize, handle: u64) -> PyResult<Vec<u8>>
```
**Explanation:**
- We pre-allocate a buffer in Rust `vec![0u8; size]`.
- The `inner.read` call decrypts the data and fills the buffer (returning the effectively read length `read_len`).
- We call `buf.truncate(read_len)` and return the vector. PyO3 automatically transforms this `Vec<u8>` into a `bytes` type object in Python.

## 4. Memory Cleanup and Persistence (`flush` & `release`)

Just as FUSE requires the correct closing of sessions, Python must clean up the handles.

```rust
fn flush(&self, handle: u64) -> PyResult<()>
fn release(&self, handle: u64) -> PyResult<()>
```
- **`flush`**: Forces the synchronization of asynchronous writes to disk (if there are still modified pages in the cache).
- **`release`**: Destroys the references associated with the `handle`. If the file is no longer used, it is removed from Rust memory, preventing a memory leak.

## Why this asynchronous-synchronous pattern?

All these internal functions call the asynchronous `tokio` system on which the FUSE base was developed, for example:
```rust
let result = self.rt.block_on(async { self.inner.write(...) })
```
This pattern guarantees an isolated runtime environment, protecting Python from the complexity of asynchronous concurrency (such as `asyncio` loops), allowing the writing of simple linear scripts with zero cognitive overhead for the end-user. At the same time, the Rust engine performs in the background, efficiently using the processor cores.
