use alloc::{string::String, sync::Arc};
use axerrno::AxResult;
use core::sync::atomic::{AtomicU64, Ordering};

use axhal::arch::{UspaceContext, TrapFrame};
use axmm::AddrSpace;
use axsync::Mutex;
use axtask::{current, AxTaskRef, CurrentTask, TaskExtRef, TaskId, TaskInner};
use crate::flags::CloneFlags;


/// Task extended data for the monolithic kernel.
pub struct TaskExt {
    /// The process ID.
    pub proc_id: u64,
    /// The parent process ID.
    pub parent_id: AtomicU64,
    /// The clear thread tid field
    ///
    /// See <https://manpages.debian.org/unstable/manpages-dev/set_tid_address.2.en.html#clear_child_tid>
    ///
    /// When the thread exits, the kernel clears the word at this address if it is not NULL.
    clear_child_tid: AtomicU64,
    /// The user space context.
    pub uctx: UspaceContext,
    /// The virtual memory address space.
    pub aspace: Arc<Mutex<AddrSpace>>,
}

impl TaskExt {
    pub const fn new(uctx: UspaceContext, aspace: Arc<Mutex<AddrSpace>>) -> Self {
        Self {
            proc_id: 233,
            parent_id: AtomicU64::new(1),
            uctx,
            clear_child_tid: AtomicU64::new(0),
            aspace,
        }
    }
    
    pub fn clone_task(
        &self,
        flags: usize,
        stack: Option<usize>,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> AxResult<u64> {
        let clone_flags = CloneFlags::from_bits((flags & !0x3f) as u32).unwrap();
        let aspace = self.aspace.clone();
        let uctx = self.uctx;
        
        let process_id = if clone_flags.contains(CloneFlags::CLONE_THREAD) {
            self.proc_id
        } else {
            TaskId::new().as_u64()
        };

        let parent_id = if clone_flags.contains(CloneFlags::CLONE_PARENT) {
            self.get_parent()
        } else {
            self.proc_id
        };

        let mut new_task = TaskInner::new(
            || {},
            String::from("cloned"),
            crate::config::KERNEL_STACK_SIZE
        );
        
        new_task.ctx_mut().set_page_table_root(aspace.lock().page_table_root());
        new_task.init_task_ext(TaskExt::new(uctx, aspace));

        let return_id: u64 = new_task.id().as_u64();

        let current_task = current();

        let mut trap_frame = 
            read_trapframe_from_kstack(current_task.get_kernel_stack_top().unwrap());
        trap_frame.set_ret_code(0);

        if let Some(stack) = stack {
            trap_frame.set_user_sp(stack);
        }
        write_trapframe_to_kstack(new_task.get_kernel_stack_top().unwrap(), &trap_frame);

        axtask::spawn_task(new_task);

        Ok(return_id)
        
    }

    pub(crate) fn clear_child_tid(&self) -> u64 {
        self.clear_child_tid
            .load(core::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn set_clear_child_tid(&self, clear_child_tid: u64) {
        self.clear_child_tid
            .store(clear_child_tid, core::sync::atomic::Ordering::Relaxed);
    }

    pub(crate) fn get_parent(&self) -> u64 {
        self.parent_id.load(Ordering::Acquire)
    }
}

axtask::def_task_ext!(TaskExt);

pub fn spawn_user_task(aspace: Arc<Mutex<AddrSpace>>, uctx: UspaceContext) -> AxTaskRef {
    let mut task = TaskInner::new(
        || {
            let curr = axtask::current();
            let kstack_top = curr.kernel_stack_top().unwrap();
            info!(
                "Enter user space: entry={:#x}, ustack={:#x}, kstack={:#x}",
                curr.task_ext().uctx.get_ip(),
                curr.task_ext().uctx.get_sp(),
                kstack_top,
            );
            unsafe { curr.task_ext().uctx.enter_uspace(kstack_top) };
        },
        "userboot".into(),
        crate::config::KERNEL_STACK_SIZE,
    );
    task.ctx_mut()
        .set_page_table_root(aspace.lock().page_table_root());
    task.init_task_ext(TaskExt::new(uctx, aspace));
    axtask::spawn_task(task)
}

pub fn write_trapframe_to_kstack(kstack_top: usize, trap_frame: &TrapFrame) {
    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let trap_frame_ptr = (kstack_top - trap_frame_size) as *mut TrapFrame;
    unsafe {
        *trap_frame_ptr = *trap_frame;
    }
}

pub fn read_trapframe_from_kstack(kstack_top: usize) -> TrapFrame {
    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let trap_frame_ptr = (kstack_top - trap_frame_size) as *mut TrapFrame;
    unsafe { *trap_frame_ptr }
}
