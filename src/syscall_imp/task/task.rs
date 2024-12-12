use axtask::TaskExtRef;

use crate::syscall_body;

pub(crate) fn sys_clone(
    flags: usize, 
    user_stack: usize,
    ptid: usize,
    arg3: usize,
    arg4: usize
) -> isize {
    syscall_body!(sys_clone, {
        let tls = arg3;
        let ctid = arg4;
    
        let stack = if user_stack == 0 {
            None
        } else {
            Some(user_stack)
        };
    
        let curr_task = axtask::current();
    
        if let Ok(new_task_id) = curr_task.task_ext().clone_task(flags, stack, ptid, tls, ctid) {
            Ok(new_task_id as isize) 
        } else {
            Err(axerrno::LinuxError::ENOMEM)
        }
    })
    
}