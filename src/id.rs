use windows::core::PCSTR;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Id {
    Integer(u16),
    Text(String)
}

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

impl From<u16> for Id {
    fn from(v: u16) -> Self {
        Id::Integer(v as u16)
    }
}

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
