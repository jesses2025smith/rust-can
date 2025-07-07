use rs_can::CanError;
use std::{
    ffi::{c_char, CStr},
    path::PathBuf,
};

#[cfg(all(target_os = "windows", target_arch = "x86"))]
const LIB_PATH: &str = "windows/x86/";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const LIB_PATH: &str = "windows/x86_64/";

#[cfg(all(target_os = "linux", target_arch = "x86"))]
const LIB_PATH: &str = "linux/x86/";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const LIB_PATH: &str = "linux/x86_64/";

#[inline]
pub fn c_str_to_string(src: *const c_char) -> Result<String, CanError> {
    if src.is_null() {
        Err(CanError::other_error("null pointer error"))
    } else {
        let c_str = unsafe { CStr::from_ptr(src) };
        let s_slice = c_str
            .to_str()
            .map_err(|e| CanError::OtherError(e.to_string()))?;
        let value = String::from(s_slice);

        Ok(value)
    }
}

#[inline]
pub(crate) fn get_libpath(path: &PathBuf, libname: &str) -> PathBuf {
    let mut path = path.clone();
    path.push(LIB_PATH);
    path.push(&libname);
    rsutil::trace!("absolute library path: {:?}", std::fs::canonicalize(&path));
    path
}
