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


pub fn utf16sz_to_u16vec(text: &String) -> Vec<u16> {
    let len = text.len()+1;
    let mut vec: Vec<u16> = Vec::with_capacity(len);
    vec.resize(len,0);
    for c in text.chars() {
        // TODO - proper encoding
        // let buf = [0;2];
        // c.encode_utf16(&mut buf);
        vec.push(c as u16);
    }
    vec.push(0);
    vec
}

pub fn utf16sz_to_u8vec(text: &String) -> Vec<u8> {
    let len = text.len()+1;
    let mut u16vec: Vec<u16> = Vec::with_capacity(len);
    // u16vec.resize(len,0);
    for c in text.chars() {
        // TODO - proper encoding
        // let buf = [0;2];
        // c.encode_utf16(&mut buf);
        u16vec.push(c as u16);
    }
    u16vec.push(0);
    let len = len*2;
    let mut u8vec = Vec::with_capacity(len);
    u8vec.resize(len,0);
    let src = unsafe { std::mem::transmute(u16vec.as_ptr()) };
    let dest = u8vec[0..].as_mut_ptr();
    unsafe { std::ptr::copy(src,dest,len); }
    u8vec
}

pub fn u32slice_to_u8vec(u32slice: &[u32]) -> Vec<u8> {
    let len = u32slice.len()*4;
    let mut u8vec = Vec::with_capacity(len);
    u8vec.resize(len,0);
    let src = unsafe { std::mem::transmute(u32slice.as_ptr()) };
    let dest = u8vec[0..].as_mut_ptr();
    unsafe { std::ptr::copy(src,dest,len); }
    u8vec
}

pub fn u32msls_as_u16vec(ms: u32, ls: u32) -> [u16;4] {
    [(ms >> 16) as u16, (ms & 0xffff) as u16, (ls >> 16) as u16, (ls & 0xffff) as u16]
}

pub fn u16vec_to_u32msls(v : &[u16;4], ms: &mut u32, ls: &mut u32) {
    *ms = (v[0] as u32) << 16 | (v[1] as u32);
    *ls = (v[2] as u32) << 16 | (v[3] as u32);
}


pub fn format_version_string(v: &[u16;4]) -> String {
    if v[3] == 0 { 
        format!("{}.{}.{}",v[0],v[1],v[2]) 
    } else {
        format!("{}.{}.{}.{}",v[0],v[1],v[2],v[3]) 
    }

}