use alloc::string::ToString;
use alloc::vec::Vec;
use core::ffi::c_char;
use arceos_posix_api::raw_ptr_to_ref_str;
use axstd::thread::yield_now;
use axtask::{current, TaskExtRef};

use crate::{ctypes::{WaitFlags, WaitStatus}, syscall_body, task::wait_pid};

/// clone() system calls create a new ("child") process.
/// By contrast with fork(2), these system calls provide more precise
/// control over what pieces of execution context are shared between
/// the calling process and the child process.
/// 
/// # Arguments
/// * `flags` - clone flags
/// * `user_stack` - pointer to lowest byte of stack
/// * `ptid` - parent thread id
/// * `tls` - location of new tls
/// * `ctid` - child thread id
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

/// wait4() system calls are similar to waitpid,
/// but additionally return resource usage information about the child
///
/// # Arguments
/// * `pid` - process id
/// * `exit_code_ptr` - exit status
/// * `option` - wait option flags
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

pub fn sys_execve(path: *const c_char, mut argv: *const usize, mut envp: *const usize) -> isize {
    let mut args_vec = Vec::new();
    let mut envs_vec = Vec::new();
    let path_str = match arceos_posix_api::char_ptr_to_str(path) {
        Ok(path) => path,
        Err(err) => {
            warn!("Failed to convert path to str: {:?}", err);
            return -1;
        }
    };

    if path_str.split('/').filter(|s| !s.is_empty()).count() > 1 {
        info!("Multi-level directories are not supported");
        return -1;
    }

    let argv_valid = unsafe { argv.is_null() || *argv == 0 };
    let envp_valid = unsafe { envp.is_null() || *envp == 0 };

    if !argv_valid {
        loop {
            let args_str_ptr = unsafe { *argv };
            if args_str_ptr == 0 {
                break;
            }
            args_vec.push(unsafe { raw_ptr_to_ref_str(args_str_ptr as *const u8) }.to_string());
            unsafe {
                argv = argv.add(1);
            }
        }
        info!("args: {:?}", args_vec);
    }

    if !envp_valid {
        loop {
            let envp_str_ptr = unsafe { *envp };
            if envp_str_ptr == 0 {
                break;
            }
            envs_vec.push(unsafe { raw_ptr_to_ref_str(envp_str_ptr as *const u8) }.to_string());
            unsafe {
                envp = envp.add(1);
            }
        }
        info!("envs: {:?}", envs_vec);
    }
    
    match crate::task::exec(path_str, args_vec, &envs_vec) {
        Ok(_) => {
            unreachable!("exec should not return");
        },
        Err(err) => {
            error!("Failed to exec: {:?}", err);
            -1
        }
    }
}