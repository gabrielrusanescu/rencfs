#![cfg(target_os = "linux")]
mod linux_mount_setup;
use linux_mount_setup::{TestGuard, DATA_PATH, MOUNT_PATH};
use serial_test::serial; // <--- ADDED
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    thread,
    time::Duration,
};

#[test]
#[serial] // <--- ADDED
fn it_mount() {
    let _guard = TestGuard::setup();
    let exists = fs::exists(Path::new(MOUNT_PATH));
    assert!(exists.is_ok(), "failed on mount {}", exists.err().unwrap());
}

#[test]
#[serial] // <--- ADDED
fn it_create_and_write_file() {
    let _guard = TestGuard::setup();
    let test_file = format!("{}/demo.txt", MOUNT_PATH);
    let path = Path::new(&test_file);
    {
        let res = File::create_new(path);
        assert!(res.is_ok(), "failed to create [{}]", res.err().unwrap());
        let _ = res.unwrap().write_all(b"test");

        let metadata = fs::metadata(path).expect("failed to retrieve metadata");
        assert!(metadata.is_file());

        let inode_dir = format!("{}/inodes/", DATA_PATH);
        let mut found = false;
        for _ in 0..50 {
            if let Ok(dir) = fs::read_dir(&inode_dir) {
                if dir.count() > 0 {
                    found = true;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(200));
        }
        assert!(found, "No inode file was created in the data folder");
    }
    let _ = fs::remove_file(path);
}

#[test]
#[serial] // <--- ADDED
fn it_create_and_rename_file() {
    let _guard = TestGuard::setup();
    let test_file1 = format!("{}/demo1.txt", MOUNT_PATH);
    let test_file2 = format!("{}/demo2.txt", MOUNT_PATH);
    {
        let _ = File::create_new(Path::new(&test_file1));
        let _ = fs::rename(Path::new(&test_file1), Path::new(&test_file2));
    }
    let _ = fs::remove_file(Path::new(&test_file2));
}

#[test]
#[serial] // <--- ADDED
fn it_create_write_rename_read_delete() {
    let _guard = TestGuard::setup();
    let test_folder = format!("{}/random", MOUNT_PATH);
    let test_file1 = format!("{}/random/initial.txt", MOUNT_PATH);
    let test_file1_renamed = format!("{}/random/renamed.txt", MOUNT_PATH);
    let test_file2 = format!("{}/random/another.txt", MOUNT_PATH);

    let tf_path = Path::new(&test_folder);
    let f1_path = Path::new(&test_file1);
    let f2_path = Path::new(&test_file2);

    let _ = fs::create_dir(tf_path);
    let mut file_handle1 = File::create_new(f1_path).unwrap();
    let _ = file_handle1.write_all(b"the quick brown fox jumps over the lazy dog");

    let _ = fs::rename(f1_path, Path::new(&test_file1_renamed));
    let _ = File::create_new(f2_path);

    let _ = fs::remove_dir_all(tf_path);
}

#[test]
#[serial] // <--- ADDED
fn it_create_empty_dir_check_attr() {
    let _guard = TestGuard::setup();
    let test_folder = format!("{}/chk_attr", MOUNT_PATH);
    let tfd_path = Path::new(&test_folder);

    let _ = fs::create_dir(tfd_path);

    let mut success = false;
    for _ in 0..50 {
        if let Ok(metadata) = fs::metadata(tfd_path) {
            if metadata.is_dir() {
                success = true;
                break;
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    assert!(
        success,
        "Timed out waiting for directory to exist/be accessible."
    );

    let inode_dir = format!("{}/inodes/", DATA_PATH);
    let mut found = false;
    if let Ok(dir) = fs::read_dir(&inode_dir) {
        if dir.count() > 0 {
            found = true;
        }
    }
    assert!(
        found,
        "No inode file (directory entry) was created in data folder"
    );

    let _ = fs::remove_dir_all(tfd_path);
}
