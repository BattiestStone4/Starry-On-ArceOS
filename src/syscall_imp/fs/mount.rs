use alloc::{boxed::Box, string::ToString};
use arceos_posix_api::AT_FDCWD;
use axerrno::AxError;

// mount file system.
// special: mount device
// dir: mount point
// fstype: file system type
// flags: mount argument
// data: argument in string form, can be NULL.

pub(crate) fn sys_mount(
    special: *const u8, 
    dir: *const u8, 
    fstype: *const u8,
    _flags: u64,
    _data: *const u8
) -> i64 {
    let result = (|| {
        let special_path = arceos_posix_api::handle_file_path(AT_FDCWD, Some(special), false)
            .inspect_err(|err| log::error!("mount: special: {:?}", err))?;

        if special_path.is_dir() {
            log::debug!("mount: special is a directory");
            return Err(AxError::InvalidInput);
        }

        let dir_path = arceos_posix_api::handle_file_path(AT_FDCWD, Some(dir), false)
            .inspect_err(|err| log::error!("mount: dir: {:?}", err))?;

        let fstype_str = arceos_posix_api::char_ptr_to_str(fstype as *const i8)
            .inspect_err(|err| log::error!("mount: fstype: {:?}", err))
            .map_err(|_| AxError::InvalidInput)?;

        if fstype_str != "vfat" {
            log::debug!("mount: fstype is not axfs");
            return Err(AxError::InvalidInput);
        }

        let dir_path_str: &'static str = Box::leak(Box::new(dir_path.to_string()));
        axfs::mount(&special_path, dir_path_str)
            .inspect_err(|err| log::error!("mount: {:?}", err))?;
        Ok(())
    })();

    match result {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// unmount file system.
// input: unmount directory, unmount argument
// return 0 if success, else return -1.
pub(crate) fn sys_umount2(special: *const u8, _flags: i32) -> i64 {
    let result = (|| {
        let special_path = arceos_posix_api::handle_file_path(AT_FDCWD, Some(special), false)
            .inspect_err(|err| log::error!("umount2: special: {:?}", err))?;
        if special_path.is_dir() {
            log::debug!("umount2: Special is a directory");
            return Err(AxError::InvalidInput);
        }

        axfs::umount(&special_path)
            .inspect_err(|err| log::error!("umount2: {:?}", err))?;

        Ok(())
    })();

    match result {
        Ok(_) => 0,
        Err(_) => -1,
    }
}