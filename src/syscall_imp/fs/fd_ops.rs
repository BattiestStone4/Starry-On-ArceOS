use core::ffi::c_int;

use arceos_posix_api as api;

/// The dup() system call allocates a new file descriptor that refers
/// to the same open file description as the descriptor oldfd.
///
/// # Arguments
/// * `old_fd` - old file descriptor
pub(crate) fn sys_dup(old_fd: c_int) -> c_int {
    api::sys_dup(old_fd)
}

/// The dup3() system call adjusts newfd the same open file descriptor as oldfd
///
/// # Arguments
/// * `old_fd` - old file descriptor
/// * `new_fd` - new file descriptor
pub(crate) fn sys_dup3(old_fd: c_int, new_fd: c_int) -> c_int {
    api::sys_dup2(old_fd, new_fd)
}

/// close() closes a file descriptor, so that it no longer refers to
/// any file and may be reused.
///
/// # Arguments
/// * `fd` - file descriptor
pub(crate) fn sys_close(fd: c_int) -> c_int {
    api::sys_close(fd)
}