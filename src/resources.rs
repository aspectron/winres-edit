use std::{fmt, path::PathBuf, sync::{Arc, Mutex}};
use windows::{
    core::PCSTR,
    Win32::Foundation::{HANDLE,HINSTANCE,BOOL}, 
    Win32::System::LibraryLoader::*, 
    // Win32::Foundation::*, 
};
use crate::utils::*;
use crate::result::*;

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

pub mod rt {
    use super::Id;
    pub const UNKNOWN: Id = Id::Integer(0);
    pub const ACCELERATOR: Id = Id::Integer(9);
    pub const ANICURSOR: Id = Id::Integer(21);
    pub const ANIICON: Id = Id::Integer(22);
    pub const BITMAP: Id = Id::Integer(2);
    pub const CURSOR: Id = Id::Integer(1);
    pub const DIALOG: Id = Id::Integer(5);
    pub const DLGINCLUDE: Id = Id::Integer(17);
    pub const FONT: Id = Id::Integer(8);
    pub const FONTDIR: Id = Id::Integer(7);
    pub const HTML: Id = Id::Integer(23);
    pub const ICON: Id = Id::Integer(3);
    pub const MANIFEST: Id = Id::Integer(24);
    pub const MENU: Id = Id::Integer(4);
    pub const MESSAGETABLE: Id = Id::Integer(11);
    pub const PLUGPLAY: Id = Id::Integer(19);
    pub const VERSION: Id = Id::Integer(16);
    pub const VXD: Id = Id::Integer(20);
}


#[derive(Debug, Clone)]
pub struct ResourceDataInner {

}

#[derive(Debug, Clone)]
pub enum ResourceData {
    Accelerator(ResourceDataInner),
    AniCursor(ResourceDataInner),
    AniIcon(ResourceDataInner),
    Bitmap(ResourceDataInner),
    Cursor(ResourceDataInner),
    Dialog(ResourceDataInner),
    DialogInclude(ResourceDataInner),
    Font(ResourceDataInner),
    FontDirectory(ResourceDataInner),
    Html(ResourceDataInner),
    Icon(ResourceDataInner),
    Manifest(ResourceDataInner),
    Menu(ResourceDataInner),
    MessageTable(ResourceDataInner),
    PlugPlay(ResourceDataInner),
    Version(ResourceDataInner),
    VxD(ResourceDataInner),
    Unknown(ResourceDataInner),
}

#[derive(Clone)]
pub struct Resource {
    pub kind : Id,
    pub name : Id,
    pub lang : u16,
    pub data : Vec<u8>,
    pub resource : ResourceData,
}

impl std::fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        f.debug_struct("")
            .field("kind",&self.kind)
            .field("name",&self.name)
            .field("lang",&self.lang)
            .field("data len",&self.data.len())
            .field("resource",&self.resource)
            //  .field("resource",&format!("{:?}",self.resource))
            //  .field("\n\tdata",&format!("[{}]: {:?}",self.data.len(),&self.data[0..std::cmp::min(30,self.data.len())]))
            //  .field("\n\ttext",&format!("{}",text.unwrap_or("N/A".into())))
            .finish()
    }
}

impl Resource {
    pub fn new(rtype : PCSTR, rname: PCSTR, rlang: u16, data : &[u8]) -> Resource {
        let typeid : Id = rtype.into();
        let resource_data = ResourceDataInner {};
        let resource = match typeid {
            rt::ACCELERATOR => ResourceData::Accelerator(resource_data),
            rt::ICON => ResourceData::Icon(resource_data),
            _ => ResourceData::Unknown(resource_data),
        };

        let info = Resource {
            kind : typeid,
            name : rname.into(),
            lang: rlang,
            data : data.to_vec(),
            resource
        };

        info
    }
}

#[derive(Debug)]
pub struct Resources {
    file : PathBuf,
    // handle : Arc<Mutex<Option<HANDLE>>>,
    // pub list : Arc<Mutex<Vec<Resource>>>,
    handle : Option<HANDLE>,
    pub list : Vec<Resource>,
}

impl Resources {

    pub fn new(file: &PathBuf) -> Resources {
        Resources {
            file : file.clone(),
            handle : None, //Arc::new(Mutex::new(None)),
            list : Vec::new(), //Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn load(&self) -> Result<()> {

        unsafe {
            let handle = LoadLibraryExA(
                pcstr!(self.file.to_str().unwrap()),
                None,
                // LOAD_LIBRARY_FLAGS::default()
                DONT_RESOLVE_DLL_REFERENCES | LOAD_LIBRARY_AS_DATAFILE
            )?;

            let ptr : *mut Resources = std::mem::transmute(&self);
            let success = EnumResourceTypesA(
                handle,
                Some(enum_types),
                std::mem::transmute(ptr)
            ).as_bool();

            FreeLibrary(handle);

            if !success {
                return Err(format!("Resources::load(): Error enumerating resources: {:?}",get_last_error()).into());
            } 
        }

        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.handle.is_some()
        // self.handle.lock().unwrap().is_some()
    }

    pub fn open(&mut self) -> Result<&Self> {
        self.open_impl(false)
    }

    pub fn open_delete_existing_resources(&mut self) -> Result<&Self> {
        self.open_impl(true)
    }

    fn open_impl(&mut self, delete_existing_resources: bool) -> Result<&Self> {
        if self.is_open() {
            return Err(format!("resource '{}' is already open", self.file.to_str().unwrap()).into());
        }

        self.load()?;

        let handle = unsafe {
            BeginUpdateResourceA(
                pcstr!(self.file.to_str().unwrap()),
                delete_existing_resources)?
        };

        self.handle.replace(handle);
        // self.handle.lock().unwrap().replace(handle);
        
        Ok(self)
    }

    pub fn remove(&self, resource: &Resource) -> Result<&Self> {
        self.remove_with_args(&resource.kind, &resource.name, resource.lang)?;
        Ok(self)
    }

    pub fn remove_with_args(&self, kind : &Id, name : &Id, lang : u16) -> Result<&Self> {
        // if let Some(handle) = self.handle.lock().unwrap().as_ref() {
        if let Some(handle) = self.handle.as_ref() {
            let success = unsafe { UpdateResourceA(
                *handle,
                kind,
                name,
                lang,
                None,
                0,
            ).as_bool() };

            if !success {
                return Err(format!("Resources::load(): Error removing resources: {:?}",get_last_error()).into());
            } 

        } else {
            return Err(format!("resource '{}' is not open", self.file.to_str().unwrap()).into());
        };

        Ok(self)
        
    }

    pub fn replace_with_args(&self, kind : &Id, name : &Id, lang : u16, data : &[u8]) -> Result<&Self> {
        if let Some(handle) = self.handle.as_ref() {
            let success = unsafe { UpdateResourceA(
                *handle,
                kind,
                name,
                lang,
                Some(std::mem::transmute(data.as_ptr())),
                data.len() as u32,
            ).as_bool() };

            if !success {
                return Err(format!("Resources::load(): Error removing resources: {:?}",get_last_error()).into());
            } 

        } else {
            return Err(format!("resource file '{}' is not open", self.file.to_str().unwrap()).into());
        };

        Ok(self)
        
    }

    pub fn close(&mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe {
                EndUpdateResourceA(handle,false);
            };
        }
    }

    pub fn discard(&mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe {
                EndUpdateResourceA(handle,true);
            };
        }
    }

    // pub fn 
    pub fn insert(&mut self, r : Resource) {
        self.list.push(r)
    }

    pub fn find(&self, typeid : Id, nameid: Id) -> Option<Resource> {
        for item in self.list.iter() {
            if item.kind == typeid && item.name == nameid {
                return Some(item.clone());
            }
        }

        return None;

    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        self.close();
    }
}

pub unsafe extern "system" fn enum_languages(hmodule: HINSTANCE, lptype: PCSTR, lpname: PCSTR, lang: u16, lparam: isize) -> BOOL {
    let rptr : *mut Resources = std::mem::transmute(lparam);
    let hresinfo = match FindResourceExA(hmodule,lptype,lpname,lang) {
        Ok(hresinfo) => hresinfo,
        Err(e) => panic!("Unable to find resource {:?} {:?} {:?} {lang}: {e}", hmodule,lptype,lpname)
    };
    let resource = LoadResource(hmodule,hresinfo);
    let len = SizeofResource(hmodule,hresinfo);
    let data_ptr = LockResource(resource);
    let data = std::slice::from_raw_parts(std::mem::transmute(data_ptr) , len as usize);
    let resources = &mut *rptr;
    resources.insert(Resource::new(lptype,lpname,lang,data));
    BOOL(1)
}

pub unsafe extern "system" fn enum_names(hmodule: HINSTANCE, lptype: PCSTR, lpname: PCSTR, lparam: isize) -> BOOL {
    EnumResourceLanguagesA(hmodule,lptype,lpname,Some(enum_languages),lparam);
    BOOL(1)
}

pub unsafe extern "system" fn enum_types(hmodule: HINSTANCE, lptype: PCSTR, lparam: isize) -> BOOL {
    EnumResourceNamesA(hmodule,lptype,Some(enum_names),lparam);
    BOOL(1)
}


