use core::ffi::c_void;

use arceos_posix_api::ctypes::rlimit;
use axerrno::LinuxResult;

use crate::ptr::UserPtr;


pub fn sys_getrlimit(resource: i32, rlim: UserPtr<c_void>) -> LinuxResult<isize> {
    warn!("sys_getrlimit: not implemented");
    Ok(0)
    // unsafe {
    //     arceos_posix_api::sys_getrlimit(resource, &rlimit)
    // }
}
