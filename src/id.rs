
use windows::core::PCSTR;

///
/// Id enum that contains a windows resource id representation.  Windows resource ids can be
/// a pointer to a string or if the pointer value is below 0xffff the id represents an integer
/// resource id.  [`Id`] encapsulates this representation into a Rust enum and provides `From`
/// and `Into` trait implementations to interop with the Windows API.
/// 
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Id {
    Integer(u16),
    Text(String)
}

/// Convert a string pointer to an `Id` 
impl From<PCSTR> for Id {
    fn from(v: PCSTR) -> Self {
        let pv = v.0 as usize;
        if pv < 0xffff {
            Id::Integer(pv as u16)
        } else {
            unsafe {
                Id::Text(v.display().to_string())
            }
        }
    }
}

/// Convert a `u16` value to an `Id`
impl From<u16> for Id {
    fn from(v: u16) -> Self {
        Id::Integer(v as u16)
    }
}

/// Convert an `Id` to a zero-terminated string pointer or
/// an integer resource representation. 
impl Into<PCSTR> for Id {
    fn into(self) -> PCSTR {
        match self {
            Id::Integer(id) => {
                PCSTR(id as *const u8)
            },
            Id::Text(text) => {
                PCSTR::from_raw(format!("{text}\0").as_ptr())
            }
        }
    }
}

impl Into<PCSTR> for &Id {
    fn into(self) -> PCSTR {
        match self {
            Id::Integer(id) => {
                PCSTR(*id as *const u8)
            },
            Id::Text(text) => {
                PCSTR::from_raw(format!("{text}\0").as_ptr())
            }
        }
    }
}
