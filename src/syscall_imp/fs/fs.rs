use core::ffi::c_char;

use arceos_posix_api as api;

/// The getcwd() function copies an absolute pathname of the current
/// working directory to the array pointed to by buf, which is of
/// length size.
///
/// # Arguments
/// * `buf` - buffer
/// * `size` - size of buffer
pub(crate) fn sys_getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    api::sys_getcwd(buf, size)
}