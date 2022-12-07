#[macro_export]
macro_rules! pcstr {
    ($s:expr) => {
        PCSTR::from_raw(format!("{}\0",$s).as_ptr())
    };
}
pub use pcstr;
use windows::{
    // Win32::System::Diagnostics::Debug::*, 
    Win32::Foundation::{WIN32_ERROR, GetLastError}, 
};

pub fn get_last_error() -> WIN32_ERROR {
    unsafe { GetLastError() }
}