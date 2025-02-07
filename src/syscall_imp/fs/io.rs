use core::ffi::{c_char, c_void};

use arceos_posix_api::{self as api, ctypes::mode_t};

/// read() attempts to read up to count bytes from file descriptor fd
/// into the buffer starting at buf.
///
/// # Arguments
/// * `fd` - file descriptor
/// * `buf` - buffer
/// * `count` - bytes read from buffer
pub(crate) fn sys_read(fd: i32, buf: *mut c_void, count: usize) -> isize {
    api::sys_read(fd, buf, count)
}

/// write() writes up to count bytes from the buffer starting at buf
/// to the file referred to by the file descriptor fd.
///
/// # Arguments
/// * `fd` - file descriptor
/// * `buf` - buffer
/// * `count` - bytes write to buffer
pub(crate) fn sys_write(fd: i32, buf: *const c_void, count: usize) -> isize {
    api::sys_write(fd, buf, count)
}

/// The writev() system call writes iovcnt buffers of data described
/// by iov to the file associated with the file descriptor fd ("gather
/// output").
///
/// # Arguments
/// * `fd` - file descriptor
/// * `iov` - iovec struct
/// * `iovcnt` - bytes write to iovec struct
pub(crate) fn sys_writev(fd: i32, iov: *const api::ctypes::iovec, iocnt: i32) -> isize {
    unsafe { api::sys_writev(fd, iov, iocnt) }
}

/// The openat() system call opens the file specified by pathname. If
/// the specified file does not exist, it may optionally (if O_CREAT
/// is specified in flags) be created by openat().
///
/// # Arguments
/// * `dirfd` - The directory file descriptor
/// * `path` - The pathname
/// * `flags` - The open flags
/// * `modes` - The access modes
pub(crate) fn sys_openat(dirfd: i32, path: *const c_char, flags: i32, modes: mode_t) -> isize {
    api::sys_openat(dirfd, path, flags, modes) as isize
}