//! Handle with signal in process.

use alloc::sync::Arc;
use axhal::arch::TrapFrame;
use axsync::Mutex;
use axtask::{current, TaskExtRef};
use crate::syscall_imp::signal::{
    signal_no::SignalNo,
    ucontext::SignalStack,
    SignalHandler, SignalSet,
};
use crate::syscall_imp::signal::ucontext::SignalUserContext;
use crate::task::{read_trapframe_from_kstack, write_trapframe_to_kstack};

pub struct SignalModule {
    /// sig_info exists or not.
    pub sig_info: bool,
    /// trapframe
    pub last_trap_frame_for_signal: Option<TrapFrame>,
    /// signal_handler
    pub signal_handler: Arc<Mutex<SignalHandler>>,
    /// signal_set
    pub signal_set: SignalSet,
    /// exit_signal
    exit_signal: Option<SignalNo>,
    /// Alternative signal stack
    pub alternate_stack: SignalStack,
}

impl SignalModule {
    /// init signal module
    pub fn init_signal(signal_handler: Option<Arc<Mutex<SignalHandler>>>) -> Self {
        let signal_handler =
            signal_handler.unwrap_or_else(|| Arc::new(Mutex::new(SignalHandler::new())));
        let signal_set = SignalSet::new();
        let last_trap_frame_for_signal = None;
        let sig_info = false;
        Self {
            sig_info,
            last_trap_frame_for_signal,
            signal_handler,
            signal_set,
            exit_signal: None,
            alternate_stack: SignalStack::default(),
        }
    }

    /// Judge whether the signal request the interrupted syscall to restart
    ///
    /// # Return
    /// - None: There is no siganl need to be delivered
    /// - Some(true): The interrupted syscall should be restarted
    /// - Some(false): The interrupted syscall should not be restarted
    pub fn have_restart_signal(&self) -> Option<bool> {
        self.signal_set.find_signal().map(|sig_num| {
            self.signal_handler
                .lock()
                .get_action(sig_num)
                .need_restart()
        })
    }

    /// Set the exit signal
    pub fn set_exit_signal(&mut self, signal: SignalNo) {
        self.exit_signal = Some(signal);
    }

    /// Get the exit signal
    pub fn get_exit_signal(&self) -> Option<SignalNo> {
        self.exit_signal
    }
}

const USER_SIGNAL_PROTECT: usize = 512;

/// load trap frame into kernel stack
/// original trap frame could be modified if SIG_INFO used.
/// return true if a trap frame which can be recovered does exist.
#[no_mangle]
pub fn load_trap_for_signal() -> bool {
    let current_task = current();

    let mut signal_modules = current_task.task_ext().signal_modules.lock();
    let signal_module = signal_modules.get_mut(&current_task.id().as_u64()).unwrap();
    if let Some(old_trap_frame) = signal_module.last_trap_frame_for_signal.take() {
        unsafe {
            let mut now_trap_frame =
                read_trapframe_from_kstack(current_task.get_kernel_stack_top().unwrap());
            let sp = now_trap_frame.get_sp();
            now_trap_frame = old_trap_frame;
            if signal_module.sig_info {
                let pc = (*(sp as *const SignalUserContext)).get_pc();
                now_trap_frame.set_pc(pc);
            }
            write_trapframe_to_kstack(
                current_task.get_kernel_stack_top().unwrap(),
                &now_trap_frame,
            );
        }
        true
    } else {
        false
    }
}