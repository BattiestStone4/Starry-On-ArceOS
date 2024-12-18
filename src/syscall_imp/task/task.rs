use axstd::thread::yield_now;
use axtask::{current, TaskExtRef};

use crate::{flags::{WaitFlags, WaitStatus}, syscall_body, task::wait_pid};

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
    
        let curr_task = current();
    
        if let Ok(new_task_id) = curr_task.task_ext().clone_task(flags, stack, ptid, tls, ctid) {
            Ok(new_task_id as isize) 
        } else {
            Err(axerrno::LinuxError::ENOMEM)
        }
    })
    
}

pub(crate) fn sys_wait4(pid: i32, exit_code_ptr: *mut i32, option: u32) -> isize {
    let option_flag = WaitFlags::from_bits(option as u32).unwrap();
    syscall_body!(sys_wait4, {
        loop {
            let answer = wait_pid(pid, exit_code_ptr);
            match answer {
                Ok(pid) => {
                    return Ok(pid as isize);
                }
                Err(status) => {
                    match status {
                        WaitStatus::NotExist => {
                            return Err(axerrno::LinuxError::ECHILD);
                        }
                        WaitStatus::Running => {
                            if option_flag.contains(WaitFlags::WNOHANG) {
                                return Ok(0);
                            }
                            else {
                                yield_now();
                            }
                        }
                        _ => {
                            panic!("Shouldn't reach here!");
                        }
                    }
                }
            }
        }
    })
}