use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};

/// Convert a string to a zero-terminated [`windows::core::PCSTR`] string.
#[macro_export]
macro_rules! pcstr {
    ($s:expr) => {
        windows::core::PCSTR::from_raw(format!("{}\0", $s).as_ptr())
    };
}
pub(crate) use pcstr;

/// Get last windows error code
pub(crate) fn get_last_error() -> WIN32_ERROR {
    unsafe { GetLastError() }
}

// /// Convert utf16 string to a zero-terminated `Vec<u16>`
// pub fn utf16sz_to_u16vec(text: &String) -> Vec<u16> {
//     let len = text.len()+1;
//     let mut vec: Vec<u16> = Vec::with_capacity(len);
//     vec.resize(len,0);
//     for c in text.chars() {
//         // TODO - proper encoding
//         // let buf = [0;2];
//         // c.encode_utf16(&mut buf);
//         vec.push(c as u16);
//     }
//     vec.push(0);
//     vec
// }

/// This function convers a string to a zero-terminated
/// `u16` unicode string represented by a `Vec<u8>` buffer.
pub(crate) fn string_to_u8vec_sz(text: &String) -> Vec<u8> {
    let len = text.len() + 1;
    let mut u16vec: Vec<u16> = Vec::with_capacity(len);
    // u16vec.resize(len,0);
    for c in text.chars() {
        // TODO - proper encoding
        // let buf = [0;2];
        // c.encode_utf16(&mut buf);
        u16vec.push(c as u16);
    }
    u16vec.push(0);
    let len = len * 2;
    let mut u8vec = vec![0; len];
    let src = unsafe { std::mem::transmute(u16vec.as_ptr()) };
    let dest = u8vec[0..].as_mut_ptr();
    unsafe {
        std::ptr::copy(src, dest, len);
    }
    u8vec
}

/// Convert `u32` (DWORD) slice to a `Vec<u8>` buffer.
pub(crate) fn u32slice_to_u8vec(u32slice: &[u32]) -> Vec<u8> {
    let len = u32slice.len() * 4;
    let mut u8vec = vec![0; len];
    let src = unsafe { std::mem::transmute(u32slice.as_ptr()) };
    let dest = u8vec[0..].as_mut_ptr();
    unsafe {
        std::ptr::copy(src, dest, len);
    }
    u8vec
}
