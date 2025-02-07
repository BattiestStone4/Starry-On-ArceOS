use core::ffi::{c_char, c_int, c_void};

use alloc::string::ToString;
use axerrno::AxError;
use arceos_posix_api::AT_FDCWD;
use axtask::{current, TaskExtRef};

use crate::syscall_body;

/// The ioctl() system call manipulates the underlying device parameters
/// of special files.
///
/// # Arguments
/// * `fd` - file descriptor
/// * `op` - request code. It is of type unsigned long in glibc and BSD,
/// and of type int in musl and other UNIX systems.
/// * `argp` - argument to the request. It is a pointer to a memory location
pub(crate) fn sys_ioctl(_fd: i32, _op: usize, _argp: *mut c_void) -> i32 {
    syscall_body!(sys_ioctl, {
        warn!("Unimplemented syscall: SYS_IOCTL");
        Ok(0)
    })
}

/// chdir() changes the current working directory of the calling
/// process to the directory specified in path.
///
/// # Arguments
/// * `path` - path name
pub(crate) fn sys_chdir(path: *const c_char) -> c_int {
    let path = match arceos_posix_api::char_ptr_to_str(path) {
        Ok(path) => path,
        Err(err) => {
            warn!("Failed to convert path: {err:?}");
            return -1;
        }
    };

    axfs::api::set_current_dir(path)
        .map(|_| 0)
        .unwrap_or_else(|err| {
            warn!("Failed to change directory: {err:?}");
            -1
        })
}

/// mkdir() attempts to create a directory named pathname.
///
/// # Arguments
/// * `dirfd` - directory file descriptor
/// * `path` - path name
/// * `mode` - directory mode
pub(crate) fn sys_mkdirat(dirfd: i32, path: *const c_char, mode: u32) -> c_int {
    let path = match arceos_posix_api::char_ptr_to_str(path) {
        Ok(path) => path,
        Err(err) => {
            warn!("Failed to convert path: {err:?}");
            return -1;
        }
    };

    if !path.starts_with("/") && dirfd != AT_FDCWD as i32 {
        warn!("unsupported.");
        return -1;
    }

    if mode != 0 {
        info!("directory mode not supported.");
    }

    axfs::api::create_dir(path)
        .map(|_| 0)
        .unwrap_or_else(|err| {
            warn!("Failed to create directory: {err:?}");
            -1
        })
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DirEnt {
    // file serial number
    d_ino: u64,
    // offset to next dirent
    d_off: i64,
    // current length of dirent
    d_reclen: u16,
    // dirent type
    d_type: u8,
    // filename string of entry
    d_name: [u8; 0],
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FileType {
    // unknown
    Unknown = 0,
    // named pipe
    Fifo = 1,
    // character device
    Chr = 2,
    // directory
    Dir = 4,
    // block device
    Blk = 6,
    // regular file
    Reg = 8,
    // symbolic link
    Lnk = 10,
    // socket
    Socket = 12,
    // whiteout
    Wht = 14,
}

impl From<axfs::api::FileType> for FileType {
    fn from(ft: axfs::api::FileType) -> Self {
        match ft {
            ft if ft.is_dir() => FileType::Dir,
            ft if ft.is_file() => FileType::Reg,
            _ => FileType::Unknown,
        }
    }
}

impl DirEnt {
    const FIXED_SIZE: usize = core::mem::size_of::<u64>() + core::mem::size_of::<i64>() + core::mem::size_of::<u16>() + core::mem::size_of::<u8>();

    fn new(ino: u64, off: i64, reclen: usize, file_type: FileType) -> Self {
        Self {
            d_ino: ino,
            d_off: off,
            d_reclen: reclen as u16,
            d_type: file_type as u8,
            d_name: [],
        }
    }

    unsafe fn write_name(&mut self, name: &[u8]) {
        core::ptr::copy_nonoverlapping(
            name.as_ptr(), 
            self.d_name.as_mut_ptr(), 
            name.len()
        );
    }
}

/// Directory buffer for getdents64 syscall
struct DirBuffer<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> DirBuffer<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    fn remaining_space(&self) -> usize {
        self.buf.len().saturating_sub(self.offset)
    }

    fn can_fit_entry(&self, entry_size: usize) -> bool {
        self.remaining_space() >= entry_size
    }

    unsafe fn write_entry(&mut self, dirent: DirEnt, name: &[u8]) -> Result<(), ()> {
        if !self.can_fit_entry(dirent.d_reclen as usize) {
            return Err(());
        }

        let entry_ptr = self.buf.as_mut_ptr().add(self.offset) as *mut DirEnt;
        entry_ptr.write(dirent);
        (*entry_ptr).write_name(name);

        self.offset += dirent.d_reclen as usize;
        Ok(())
    }
}

/// The getdents64() system call reads several linux_dirent structures
/// from the directory referred to by the open file descriptor fd into
/// the buffer pointed to by dirp.  The argument count specifies the
/// size of that buffer.
///
/// # Arguments
/// * `fd` - file descriptor
/// * `buf` - buffer
/// * `len` - length of buffer
pub(crate) fn sys_getdents64(fd: i32, buf: *mut c_void, len: usize) -> isize {
    if len < DirEnt::FIXED_SIZE {
        warn!("Buffer size too small: {len}");
        return -1;
    }

    let current_task = current();
    if let Err(e) = current_task.task_ext().aspace.lock().alloc_for_lazy((buf as usize).into(), len) {
        warn!("Memory allocation failed: {:?}", e);
        return -1;
    }

    let path = match arceos_posix_api::Directory::from_fd(fd).map(|dir| dir.path().to_string()) {
        Ok(path) => path,
        Err(err)  => {
            warn!("Invalid directory descriptor: {:?}", err);
            return -1;
        }
    };

    let mut buffer = unsafe { DirBuffer::new(core::slice::from_raw_parts_mut(buf as *mut u8, len)) };

    let (initial_offset, count) = unsafe {
        let mut buf_offset = 0;
        let mut count = 0;
        while buf_offset + DirEnt::FIXED_SIZE <= len {
            let dir_ent = *(buf.add(buf_offset) as *const DirEnt);
            if dir_ent.d_reclen == 0 { break; }

            buf_offset += dir_ent.d_reclen as usize;
            assert_eq!(dir_ent.d_off, buf_offset as i64);
            count += 1;
        }
        (buf_offset as i64, count)
    };

    axfs::api::read_dir(&path)
        .map_err(|_| -1)
        .and_then(|entries| {
            let mut total_size = initial_offset as usize;
            let mut current_offset = initial_offset;

            for entry in entries.flatten().skip(count) {
                let mut name = entry.file_name();
                name.push('\0');
                let name_bytes = name.as_bytes();

                let entry_size = DirEnt::FIXED_SIZE + name_bytes.len();
                current_offset += entry_size as i64;

                let dirent = DirEnt::new(
                    1,
                    current_offset,
                    entry_size,
                    FileType::from(entry.file_type()),
                );

                unsafe {
                    if buffer.write_entry(dirent, name_bytes).is_err() {
                        break;
                    }
                }

                total_size += entry_size;
            }

            if total_size > 0 && buffer.can_fit_entry(DirEnt::FIXED_SIZE) {
                let terminal = DirEnt::new(1, current_offset, 0, FileType::Reg);
                unsafe { let _ = buffer.write_entry(terminal, &[]); }
            }

            Ok(total_size as isize)
        })
        .unwrap_or(-1)
}

/// linkat() creates a new link (also known as a hard link) to an
/// existing file.
///
/// # Arguments
/// * `old_dirfd` - old directory file descriptor
/// * `old_path` - old file path
/// * `new_dirfd` - new directory file descriptor
/// * `new_path` - new file path
/// * `flags` - link flags
pub(crate) fn sys_linkat(
    old_dirfd: i32,
    old_path: *const u8,
    new_dirfd: i32,
    new_path: *const u8,
    flags: i32
) -> i32 {
    if flags != 0 {
        warn!("Unsupported flags: {flags}");
    }

    // handle old path
    arceos_posix_api::handle_file_path(old_dirfd as isize, Some(old_path), false)
        .inspect_err(|err| warn!("Failed to convert new path: {err:?}"))
        .and_then(|old_path| {
            //handle new path
            arceos_posix_api::handle_file_path(new_dirfd as isize, Some(new_path), false)
                .inspect_err(|err| warn!("Failed to convert new path: {err:?}"))
                .map(|new_path| (old_path, new_path))
        })
        .and_then(|(old_path, new_path)| {
            arceos_posix_api::HARDLINK_MANAGER
                .create_link(&new_path, &old_path)
                .inspect_err(|err| warn!("Failed to create link: {err:?}"))
                .map_err(Into::into)
        })
        .map(|_| 0)
        .unwrap_or(-1)
}

/// unlink() deletes a name from the filesystem.
///
/// # Arguments
/// * `dir_fd` - directory file descriptor
/// * `path` - file path
/// * `flags` - unlink flags
pub fn sys_unlinkat(dir_fd: isize, path: *const u8, flags: usize) -> isize {
    const AT_REMOVEDIR: usize = 0x200;

    arceos_posix_api::handle_file_path(dir_fd, Some(path), false)
        .inspect_err(|e| warn!("unlinkat error: {:?}", e)) 
        .and_then(|path| {
            if flags == AT_REMOVEDIR {
                axfs::api::remove_dir(path.as_str())
                    .inspect_err(|e| warn!("unlinkat error: {:?}", e)) 
                    .map(|_| 0)
            } else {
                axfs::api::metadata(path.as_str()).and_then(|metadata| {
                    if metadata.is_dir() {
                        Err(AxError::IsADirectory)
                    } else {
                        debug!("unlink file: {:?}", path);
                        arceos_posix_api::HARDLINK_MANAGER
                            .remove_link(&path)
                            .ok_or_else(|| {
                                debug!("unlink file error");
                                AxError::NotFound
                            })
                            .map(|_| 0)
                    }
                })
            }
        })
        .unwrap_or(-1)
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[cfg(not(target_arch = "x86_64"))]
pub struct Kstat {
    /// device
    pub st_dev: u64,
    /// inode number
    pub st_ino: u64,
    /// file type and mode
    pub st_mode: u32,
    /// number of hard links
    pub st_nlink: u32,
    /// user id
    pub st_uid: u32,
    /// group id
    pub st_gid: u32,
    /// device id
    pub st_rdev: u64,
    /// padding
    pub _pad0: u64,
    /// total size
    pub st_size: u64,
    /// block size
    pub st_blksize: u32,
    /// padding
    pub _pad1: u32,
    /// number of blocks
    pub st_blocks: u64,
    /// time of last access (sec)
    pub st_atime_sec: isize,
    /// time of last access (nsec)
    pub st_atime_nsec: isize,
    /// time of last modification (sec)
    pub st_mtime_sec: isize,
    /// time of last modification (nsec)
    pub st_mtime_nsec: isize,
    /// time of last status change (sec)
    pub st_ctime_sec: isize,
    /// time of last status change (nsec)
    pub st_ctime_nsec: isize,
}

impl From<arceos_posix_api::ctypes::stat> for Kstat {
    fn from(stat: arceos_posix_api::ctypes::stat) -> Self {
        Self {
            st_dev: stat.st_dev,
            st_ino: stat.st_ino,
            st_mode: stat.st_mode,
            st_nlink: stat.st_nlink,
            st_uid: stat.st_uid,
            st_gid: stat.st_gid,
            st_rdev: stat.st_rdev,
            _pad0: 0,
            st_size: stat.st_size as u64,
            st_blksize: stat.st_blksize as u32,
            _pad1: 0,
            st_blocks: stat.st_blocks as u64,
            st_atime_sec: stat.st_atime.tv_sec as isize,
            st_atime_nsec: stat.st_atime.tv_nsec as isize,
            st_mtime_sec: stat.st_mtime.tv_sec as isize,
            st_mtime_nsec: stat.st_mtime.tv_nsec as isize,
            st_ctime_sec: stat.st_ctime.tv_sec as isize,
            st_ctime_nsec: stat.st_ctime.tv_nsec as isize,
        }
    }
}

/// fstat() retrieves information about the file that is specified by the file descriptor fd.
///
/// # Arguments
/// * `fd` - file descriptor
/// * `kstatbuf` - stat buffer
pub(crate) fn sys_fstat(fd: i32, kstatbuf: *mut c_void) -> i32 {
    let kstatbuf = kstatbuf as *mut Kstat;
    let mut statbuf = arceos_posix_api::ctypes::stat::default();

    if unsafe { 
        arceos_posix_api::sys_fstat(fd, &mut statbuf as *mut arceos_posix_api::ctypes::stat) 
    } < 0 {
        return -1;
    }

    unsafe {
        let kstat = Kstat::from(statbuf);
        kstatbuf.write(kstat);
    }
    0
}