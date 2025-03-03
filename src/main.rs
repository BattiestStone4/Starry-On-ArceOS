#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate log;
extern crate alloc;
extern crate axstd;

#[rustfmt::skip]
mod config {
    include!(concat!(env!("OUT_DIR"), "/uspace_config.rs"));
}
mod loader;
mod mm;
mod syscall_imp;
mod task;
mod ctypes;
mod signal;

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use axhal::arch::UspaceContext;
use axsync::Mutex;

static VFAT_IMG: &'static [u8] = include_bytes!("../vfat.img"); //used by sys_mount

pub fn get_envs() -> Vec<String> {
    // Const string for environment variables
    let mut envs:Vec<String> = vec![
        "SHLVL=1".into(),
        "PWD=/".into(),
        "GCC_EXEC_PREFIX=/riscv64-linux-musl-native/bin/../lib/gcc/".into(),
        "COLLECT_GCC=./riscv64-linux-musl-native/bin/riscv64-linux-musl-gcc".into(),
        "COLLECT_LTO_WRAPPER=/riscv64-linux-musl-native/bin/../libexec/gcc/riscv64-linux-musl/11.2.1/lto-wrapper".into(),
        "COLLECT_GCC_OPTIONS='-march=rv64gc' '-mabi=lp64d' '-march=rv64imafdc' '-dumpdir' 'a.'".into(),
        "LIBRARY_PATH=/lib/".into(),
        "LD_LIBRARY_PATH=/lib/".into(),
        "LD_DEBUG=files".into(),
    ];
    // read the file "/etc/environment"
    // if exist, then append the content to envs
    // else set the environment variable to default value
    if let Some(environment_vars) = axfs::api::read_to_string("/etc/environment").ok() {
        envs.push(environment_vars);
    } else {
        envs.push("PATH=/usr/sbin:/usr/bin:/sbin:/bin".into());
    }
    envs
}

fn get_args(command_line: &[u8]) -> Vec<String> {
    let mut args = Vec::new();
    // Check whether quote exists
    let mut in_quote = false;
    let mut arg_start = 0; // a new args start position
    for pos in 0..command_line.len() {
        if command_line[pos] == b'\"' {
            in_quote = !in_quote;
        }
        if command_line[pos] == b' ' && !in_quote {
            // need to divide
            // prevent empty string
            if arg_start != pos {
                args.push(
                    core::str::from_utf8(&command_line[arg_start..pos])
                        .unwrap()
                        .to_string(),
                );
            }
            arg_start = pos + 1;
        }
    }
    // last args
    if arg_start != command_line.len() {
        args.push(
            core::str::from_utf8(&command_line[arg_start..])
                .unwrap()
                .to_string(),
        );
    }
    args
}


#[no_mangle]
fn main() {
    let testcases = option_env!("AX_TESTCASES_LIST")
        .unwrap_or_else(|| "Please specify the testcases list by making user_apps")
        .split(',')
        .filter(|&x| !x.is_empty());

    let _ = axfs::fops::File::open(
        "/vda2",
        &axfs::fops::OpenOptions::new()
            .set_create(true, true)
            .set_read(true)
            .set_write(true),
    )
    .inspect_err(|err| debug!("Failed to open /dev/vda2: {:?}", err))
    .and_then(|mut file| file.write(VFAT_IMG))
    .inspect_err(|err| debug!("Failed to write /dev/vda2: {:?}", err));
    
    for testcase in testcases {
        info!("Running testcase: {}", testcase);
        let args = get_args(testcase.as_bytes());
        let mut args_vec: Vec<String> = Vec::new();
        for arg in args {
            args_vec.push(arg);
        }
        let path = args_vec[0].clone();
        let envs = get_envs();
        let (entry_vaddr, ustack_top, uspace) = mm::load_user_app(&path, args_vec, &envs).unwrap();
        let user_task = task::spawn_user_task(
            Arc::new(Mutex::new(uspace)),
            UspaceContext::new(entry_vaddr.into(), ustack_top, 2333),
            0,
        );
        let exit_code = user_task.join();
        info!("User task {} exited with code: {:?}", testcase, exit_code);
    }
    
}
