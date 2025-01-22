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

use alloc::sync::Arc;

use axhal::arch::UspaceContext;
use axsync::Mutex;

static VFAT_IMG: &'static [u8] = include_bytes!("../vfat.img");

#[no_mangle]
fn main() {
    //loader::list_apps();
    let testcases = option_env!("AX_TESTCASES_LIST")
        .unwrap_or_else(|| "Please specify the testcases list by making user_apps")
        .split(',')
        .filter(|&x| !x.is_empty());

    let _ = axfs::fops::File::open(
        "/dev/vda2",
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
        let (entry_vaddr, ustack_top, uspace) = mm::load_user_app(testcase).unwrap();
        let user_task = task::spawn_user_task(
            Arc::new(Mutex::new(uspace)),
            UspaceContext::new(entry_vaddr.into(), ustack_top, 2333),
            0,
        );
        let exit_code = user_task.join();
        info!("User task {} exited with code: {:?}", testcase, exit_code);
    }
}
