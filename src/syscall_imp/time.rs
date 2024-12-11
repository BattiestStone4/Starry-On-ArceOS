use core::ffi::c_int;

use arceos_posix_api::{self as api, ctypes::timeval};

pub(crate) fn sys_clock_gettime(clock_id: i32, tp: *mut api::ctypes::timespec) -> i32 {
    unsafe { api::sys_clock_gettime(clock_id, tp) }
}

pub(crate) fn sys_get_time_of_day(ts: *mut timeval) -> c_int {
    unsafe { api::sys_get_time_of_day(ts) }
}
