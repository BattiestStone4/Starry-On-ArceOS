//! clone 任务时指定的参数。

use bitflags::*;

bitflags! {
    /// for sys_clone
    #[derive(Debug, Clone, Copy)]
    pub struct CloneFlags: u32 {
        /// .
        const CLONE_NEWTIME = 1 << 7;
        /// share memory space
        const CLONE_VM = 1 << 8;
        /// share filesystem info
        const CLONE_FS = 1 << 9;
        /// share fd table
        const CLONE_FILES = 1 << 10;
        /// share signal handler function
        const CLONE_SIGHAND = 1 << 11;
        /// create fd referring to child process, for sys_pidfd_open
        const CLONE_PIDFD = 1 << 12;
        /// for sys_ptrace
        const CLONE_PTRACE = 1 << 13;
        /// if set, calling process suspended until child releases its virtual memory
        const CLONE_VFORK = 1 << 14;
        /// specify ppid of process the same as current
        const CLONE_PARENT = 1 << 15;
        /// created as "thread"
        const CLONE_THREAD = 1 << 16;
        /// cloned child use new mount namespace
        const CLONE_NEWNS = 1 << 17;
        /// share the same semaphore, for sys_semop
        const CLONE_SYSVSEM = 1 << 18;
        /// set tls
        const CLONE_SETTLS = 1 << 19;
        /// store child thread id at the location pointed by parent_tid
        const CLONE_PARENT_SETTID = 1 << 20;
        /// zero the child thread ID
        const CLONE_CHILD_CLEARTID = 1 << 21;
        /// historical flag, defined but ignored
        const CLONE_DETACHED = 1 << 22;
        /// related to sys_trace, not used
        const CLONE_UNTRACED = 1 << 23;
        /// store child thread id at the location pointed by child_tid
        const CLONE_CHILD_SETTID = 1 << 24;
        /// New pid namespace.
        const CLONE_NEWPID = 1 << 29;
    }

    pub struct WaitFlags: u32 {
        /// return immediately if no child has exited.
        const WNOHANG = 1 << 0;
        /// return if a child has stopped
        const WIMTRACED = 1 << 1;
        /// return if a stopped child has been resumed by delivery of SIGCONT.
        const WCONTINUED = 1 << 3;
        /// Wait for any child
        const WALL = 1 << 30;
        /// Wait for cloned process
        const WCLONE = 1 << 31;
    }

}

/// sys_wait4 的返回值
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitStatus {
    /// exited
    Exited,
    /// running
    Running,
    /// not exist
    NotExist,
}
#[repr(C)]
pub struct Tms {
    /// user time in us
    pub tms_utime: usize,
    /// system time in us
    pub tms_stime: usize,
    /// user time of children in us
    pub tms_cutime: usize,
    /// system time of children in us
    pub tms_cstime: usize,
}

numeric_enum_macro::numeric_enum! {
    #[repr(i32)]
    #[allow(non_camel_case_types)]
    #[derive(Eq, PartialEq, Debug, Clone, Copy)]
    pub enum TimerType {
    /// NO TIMER
    NONE = -1,
    /// SYSTEM TIMER
    REAL = 0,
    /// USER TIMER
    VIRTUAL = 1,
    /// ALL TIMER
    PROF = 2,
    }
}

impl From<usize> for TimerType {
    fn from(num: usize) -> Self {
        match Self::try_from(num as i32) {
            Ok(val) => val,
            Err(_) => Self::NONE,
        }
    }
}
pub struct TimeStat {
    /// user time in ns
    utime_ns: usize,
    /// system time in ns
    stime_ns: usize,
    /// user timestamp
    user_timestamp: usize,
    /// kernel timestamp
    kernel_timestamp: usize,
    /// timer type
    timer_type: TimerType,
    /// timer interval in ns
    timer_interval_ns: usize,
    /// timer remained time in ns
    timer_remained_ns: usize,
}

impl Default for TimeStat {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeStat {
    pub fn new() -> Self {
        Self {
            utime_ns: 0,
            stime_ns: 0,
            user_timestamp: 0,
            kernel_timestamp: 0,
            timer_type: TimerType::NONE,
            timer_interval_ns: 0,
            timer_remained_ns: 0,
        }
    }

    pub fn output(&self) -> (usize, usize) {
        (self.utime_ns, self.stime_ns)
    }

    pub fn reset(&mut self, current_timestamp: usize) {
        self.utime_ns = 0;
        self.stime_ns = 0;
        self.user_timestamp = 0;
        self.kernel_timestamp = current_timestamp;
    }

    pub fn switch_into_kernel_mode(&mut self, current_timestamp: usize) {
        let now_time_ns = current_timestamp;
        let delta = now_time_ns - self.kernel_timestamp;
        self.utime_ns += delta;
        self.kernel_timestamp = now_time_ns;
        if self.timer_type != TimerType::NONE {
            self.update_timer(delta);
        };
    }

    pub fn switch_into_user_mode(&mut self, current_timestamp: usize) {
        let now_time_ns = current_timestamp;
        let delta = now_time_ns - self.kernel_timestamp;
        self.stime_ns += delta;
        self.user_timestamp = now_time_ns;
        if self.timer_type == TimerType::REAL || self.timer_type == TimerType::PROF {
            self.update_timer(delta);
        }
    }

    pub fn switch_from_old_task(&mut self, current_timestamp: usize) {
        let now_time_ns = current_timestamp;
        let delta = now_time_ns - self.kernel_timestamp;
        self.stime_ns += delta;
        self.kernel_timestamp = now_time_ns;
        if self.timer_type == TimerType::REAL || self.timer_type == TimerType::PROF {
            self.update_timer(delta);
        }
    }

    pub fn switch_to_new_task(&mut self, current_timestamp: usize) {
        let now_time_ns = current_timestamp;
        let delta = now_time_ns - self.kernel_timestamp;
        self.kernel_timestamp = now_time_ns;
        if self.timer_type == TimerType::REAL {
            self.update_timer(delta);
        }
    }

    pub fn set_timer(
        &mut self,
        timer_interval_ns: usize,
        timer_remained_ns: usize,
        timer_type: usize,
    ) -> bool {
        self.timer_type = timer_type.into();
        self.timer_interval_ns = timer_interval_ns;
        self.timer_remained_ns = timer_remained_ns;
        self.timer_type != TimerType::NONE
    }

    pub fn update_timer(&mut self, delta: usize) {
        if self.timer_remained_ns == 0 {
            return;
        }
        if self.timer_remained_ns > delta {
            self.timer_remained_ns -= delta;
            return;
        }
    }
}