use alloc::{boxed::Box, string::ToString};
use arceos_posix_api::AT_FDCWD;
use axerrno::AxError;

/// mount() attaches the filesystem specified by source (which is
/// often a pathname referring to a device, but can also be the
/// pathname of a directory or file, or a dummy string) to the
/// location (a directory or file) specified by the pathname in
/// target.
///
/// # Arguments
/// * `special` - pathname referring to a device
/// * `dir` - target pathname
/// * `fstype` - file system type
/// * `flags` - mount flags
/// * `data` - a string of comma-separated options understood by this filesystem
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

/// umount() remove the attachment of the (topmost)
/// filesystem mounted on target.
///
/// # Arguments
/// * `special` - pathname the file system mounting on
/// * `flags` - mount flags
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