use core::ffi::c_int;

use arceos_posix_api::{self as api, ctypes::timeval};
use axhal::time::{monotonic_time_nanos, nanos_to_ticks};

use crate::{ctypes::Tms, syscall_body, task::time_stat_output};

/// clock_gettime() retrieves and sets
/// the time of the specified clock clockid.
/// # Arguments
/// * `clockid` - specified clock id
pub(crate) fn sys_clock_gettime(clock_id: i32, tp: *mut api::ctypes::timespec) -> i32 {
    unsafe { api::sys_clock_gettime(clock_id, tp) }
}

/// gettimeofday() can get and set the time as well as a timezone.
/// # Arguments
/// * `ts` - struct timeval address
pub(crate) fn sys_get_time_of_day(ts: *mut timeval) -> c_int {
    unsafe { api::sys_get_time_of_day(ts) }
}

/// times() stores the current process times in the struct tms that
/// buf points to.
/// # Arguments
/// * `tms` - struct tms address
pub fn sys_times(tms: *mut Tms) -> isize {
    syscall_body!(sys_times, {
        let (_, utime_us, _, stime_us) = time_stat_output();
        unsafe {
            *tms = Tms {
                tms_utime: utime_us,
                tms_stime: stime_us,
                tms_cutime: utime_us,
                tms_cstime: stime_us,
            }
        }
        Ok(nanos_to_ticks(monotonic_time_nanos()) as isize)
    })
}