//! To define the signal action and its flags

use crate::syscall_imp::signal::signal_no::SignalNo::{self, *};

/// represents default handler.
pub const SIG_DFL: usize = 0;

/// represents ignore the signal.
pub const SIG_IGN: usize = 1;

bitflags::bitflags! {
    #[allow(missing_docs)]
    #[derive(Default,Clone, Copy, Debug)]
    /// The flags of the signal action
    pub struct SigActionFlags: u32 {
        /// do not receive notification when child processes stop
        const SA_NOCLDSTOP = 1;
        /// do not create zombie on child process exit
        const SA_NOCLDWAIT = 2;
        /// use signal handler with 3 arguments, and sa_sigaction should be set instead of sa_handler.
        const SA_SIGINFO = 4;
        /// call the signal handler on an alternate signal stack provided by `sigaltstack(2)`
        const SA_ONSTACK = 0x08000000;
        /// restart system calls if possible
        const SA_RESTART = 0x10000000;
        /// do not automatically block the signal when its handler is being executed
        const SA_NODEFER = 0x40000000;
        /// restore the signal action to the default upon entry to the signal handler
        const SA_RESETHAND = 0x80000000;
        /// use the restorer field as the signal trampoline
        const SA_RESTORER = 0x4000000;
    }
}

pub enum SignalDefault {
    /// Terminate
    Terminate,
    /// Ignore
    Ignore,
    /// Terminate the process and dump the core. TODO: core dump.
    Core,
    /// Stop
    Stop,
    /// restart
    Cont,
}

impl SignalDefault {
    /// Get the default action of a signal
    pub fn get_action(signal: SignalNo) -> Self {
        match signal {
            SIGABRT => Self::Core,
            SIGALRM => Self::Terminate,
            SIGBUS => Self::Core,
            SIGCHLD => Self::Ignore,
            SIGCONT => Self::Cont,
            SIGFPE => Self::Core,
            SIGHUP => Self::Terminate,
            SIGILL => Self::Core,
            SIGINT => Self::Terminate,
            SIGKILL => Self::Terminate,
            SIGPIPE => Self::Terminate,
            SIGQUIT => Self::Core,
            SIGSEGV => Self::Core,
            SIGSTOP => Self::Stop,
            SIGTERM => Self::Terminate,
            SIGTSTP => Self::Stop,
            SIGTTIN => Self::Stop,
            SIGTTOU => Self::Stop,
            SIGUSR1 => Self::Terminate,
            SIGUSR2 => Self::Terminate,
            SIGXCPU => Self::Core,
            SIGXFSZ => Self::Core,
            SIGVTALRM => Self::Terminate,
            SIGPROF => Self::Terminate,
            SIGWINCH => Self::Ignore,
            SIGIO => Self::Terminate,
            SIGPWR => Self::Terminate,
            SIGSYS => Self::Core,
            _ => Self::Terminate,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
/// The structure of the signal action
pub struct SigAction {
    /// Address of signal handler
    /// 1. if SIG_DFL || SIG_IGN, handle as description says.
    /// 2. flags without SA_SIGINFO, then fn(sig: SignalNo) -> ()，the same as void (*sa_handler)(int) in C.
    /// 3. flags with SA_SIGINFO, then fn(sig: SignalNo, info: &SigInfo, ucontext: &mut UContext) -> ().
    /// the same as void (*sa_sigaction)(int, siginfo_t *, void *) in C.
    pub sa_handler: usize,
    /// sa_flags
    pub sa_flags: SigActionFlags,
    /// restorer
    pub restorer: usize,
    /// sa_mask
    pub sa_mask: usize,
}

impl SigAction {
    /// get the restorer address of the signal action
    ///
    /// When the SA_RESTORER flag is set, the restorer address is valid
    ///
    /// or it will return None, and the core will set the restore address as the signal trampoline
    pub fn get_storer(&self) -> Option<usize> {
        if self.sa_flags.contains(SigActionFlags::SA_RESTORER) {
            Some(self.restorer)
        } else {
            None
        }
    }

    /// Whether the syscall should be restarted after the signal handler returns
    pub fn need_restart(&self) -> bool {
        self.sa_flags.contains(SigActionFlags::SA_RESTART)
    }
}