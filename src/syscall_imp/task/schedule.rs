use arceos_posix_api as api;

/// sched_yield() causes the calling thread to relinquish the CPU.
pub(crate) fn sys_sched_yield() -> i32 {
    api::sys_sched_yield()
}

/// nanosleep() suspends the execution of the calling thread until
/// either at least the time specified in *duration has elapsed, or
/// the delivery of a signal that triggers the invocation of a handler
/// in the calling thread or that terminates the process.
///
/// # Arguments
/// * `req` - duration
/// * `rem` - can then be used to call nanosleep() again and complete the specified pause
pub(crate) fn sys_nanosleep(
    req: *const api::ctypes::timespec,
    rem: *mut api::ctypes::timespec,
) -> i32 {
    unsafe { api::sys_nanosleep(req, rem) }
}
