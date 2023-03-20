use crate::id::*;
use crate::result::*;
use crate::utils::*;
use crate::version::*;
use std::path::Path;
use std::{
    fmt,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use windows::{
    core::PCSTR,
    Win32::Foundation::{BOOL, HANDLE, HINSTANCE},
    Win32::System::LibraryLoader::*,
};

pub mod resource_type {
    //!
    //! List of resource constants representing Windows resource types
    //! expressed as [`Id`]
    //!
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

/// Placeholder for future data serialization (not implementated)
#[derive(Debug, Clone)]
pub struct ResourceDataInner {
    // ...
}

/// Placeholder for future data serialization (not implementated)
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
    Version(VersionInfo),
    VxD(ResourceDataInner),
    Unknown(ResourceDataInner),
}

/// Structure representing a single resource
#[derive(Clone)]
pub struct Resource {
    /// resource type
    pub kind: Id,
    /// resource name
    pub name: Id,
    /// `u16` language associated with the resource
    pub lang: u16,
    /// raw resource data
    pub encoded: Arc<Mutex<Vec<u8>>>,
    /// destructured resource data (not implemented)
    pub decoded: Arc<Mutex<Option<ResourceData>>>,
    /// reference to the module handle that owns the resource
    module_handle: Arc<Mutex<Option<HANDLE>>>,
}

impl std::fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("")
            .field("kind", &self.kind)
            .field("name", &self.name)
            .field("lang", &self.lang)
            .field("data len", &self.encoded.lock().unwrap().len())
            // .field("resource",&self.decoded)
            //  .field("resource",&format!("{:?}",self.resource))
            .finish()
    }
}

impl Resource {
    /// Create a new resource instance bound to the [`Resources`] resource manager.
    pub fn new(
        resources: &Resources,
        rtype: PCSTR,
        rname: PCSTR,
        rlang: u16,
        data: &[u8],
    ) -> Resource {
        let typeid: Id = rtype.into();
        Resource {
            kind: typeid,
            name: rname.into(),
            lang: rlang,
            encoded: Arc::new(Mutex::new(data.to_vec())),
            decoded: Arc::new(Mutex::new(None)),
            module_handle: resources.module_handle(),
        }
    }

    /// Remove resource from the associated module (deletes the resource)
    pub fn remove(&self) -> Result<&Self> {
        if let Some(handle) = self.module_handle.lock().unwrap().as_ref() {
            let success = unsafe {
                UpdateResourceA(
                    *handle,
                    self.kind.clone(),
                    self.name.clone(),
                    self.lang,
                    None,
                    0,
                )
                .as_bool()
            };

            if !success {
                return Err(format!(
                    "Resources::load(): Error removing resources: {:?}",
                    get_last_error()
                )
                .into());
            }
        } else {
            return Err("Resource::replace(): resource file is not open".into());
        };

        Ok(self)
    }

    /// Replace raw resource data with a user-supplied data. This only replaces
    /// the data in the resource structure. You must call [`Resource::update()`]
    /// following this call to update the resoruce data in the actual module.
    pub fn replace(&self, data: &[u8]) -> Result<&Self> {
        *self.encoded.lock().unwrap() = data.to_vec();
        Ok(self)
    }

    /// Store this resource in the resource module (creates new or updates)
    pub fn update(&self) -> Result<&Self> {
        if let Some(handle) = self.module_handle.lock().unwrap().as_ref() {
            let encoded = self.encoded.lock().unwrap();
            let success = unsafe {
                UpdateResourceA(
                    *handle,
                    self.kind.clone(),
                    self.name.clone(),
                    self.lang,
                    Some(std::mem::transmute(encoded.as_ptr())),
                    encoded.len() as u32,
                )
                .as_bool()
            };

            if !success {
                return Err(format!(
                    "Resources::load(): Error removing resources: {:?}",
                    get_last_error()
                )
                .into());
            }
        } else {
            return Err("Resource::replace(): resource file is not open".into());
        };

        Ok(self)
    }
}

/// Data structure representing a resource file. This data structure
/// points to a `.res` or `.exe` file and allows loading and modifying
/// resource in this file.
#[derive(Debug)]
pub struct Resources {
    file: PathBuf,
    module_handle: Arc<Mutex<Option<HANDLE>>>,
    /// resources contained in the supplied file represented by the [`Resource`] data structure.
    pub list: Arc<Mutex<Vec<Arc<Resource>>>>,
}

impl Resources {
    /// Create new instance of the resource manager bound to a specific resource file.
    /// Once created, the resource file should be opened using [`open()`] or [`load()`].
    pub fn new(file: &Path) -> Resources {
        Resources {
            file: file.to_path_buf(),
            module_handle: Arc::new(Mutex::new(None)),
            list: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Load resources from the resource file.  This function does not need to be called
    /// explicitly as [`Resources::open`] will call it. It is useful if you want to load
    /// resources for extraction purposes only.
    pub fn load(&self) -> Result<()> {
        unsafe {
            let handle = LoadLibraryExA(
                pcstr!(self.file.to_str().unwrap()),
                None,
                // LOAD_LIBRARY_FLAGS::default()
                DONT_RESOLVE_DLL_REFERENCES | LOAD_LIBRARY_AS_DATAFILE,
            )?;

            let ptr: *const Resources = std::mem::transmute(&*self);
            let success =
                EnumResourceTypesA(handle, Some(enum_types), std::mem::transmute(ptr)).as_bool();

            FreeLibrary(handle);

            if !success {
                return Err(format!(
                    "Resources::load(): Error enumerating resources: {:?}",
                    get_last_error()
                )
                .into());
            }
        }

        Ok(())
    }

    pub fn module_handle(&self) -> Arc<Mutex<Option<HANDLE>>> {
        self.module_handle.clone()
    }

    /// returns `true` if the resource file is currently open
    pub fn is_open(&self) -> bool {
        self.module_handle.lock().unwrap().is_some()
    }

    /// Open the resource file. This function opens a Windows handle to
    /// the resource file and must be followed by [`Resources::close`].
    pub fn open(&mut self) -> Result<&Self> {
        self.open_impl(false)
    }

    /// Opens the resource file with `delete_existing_resources` set to `true`.
    /// This will result in retention in a deletion of previously existing resources.
    pub fn open_delete_existing_resources(&mut self) -> Result<&Self> {
        self.open_impl(true)
    }

    fn open_impl(&mut self, delete_existing_resources: bool) -> Result<&Self> {
        if self.is_open() {
            return Err(
                format!("resource '{}' is already open", self.file.to_str().unwrap()).into(),
            );
        }

        self.load()?;

        let handle = unsafe {
            BeginUpdateResourceA(
                pcstr!(self.file.to_str().unwrap()),
                delete_existing_resources,
            )?
        };

        self.module_handle.lock().unwrap().replace(handle);
        // self.handle.lock().unwrap().replace(handle);

        Ok(self)
    }

    /// Remove the supplied resource from the resource file.
    pub fn remove(&self, resource: &Resource) -> Result<&Self> {
        self.remove_with_args(&resource.kind, &resource.name, resource.lang)?;
        Ok(self)
    }

    /// Remove the resource from the resource file by specifying resource type, name and lang.
    /// WARNING: If this method fails, the entire update set may fail (this is true for any API calls).
    /// As such it is highly recommended to use [`Resources::remove`] instead and supplying and existing
    /// [`Resource`] struct as it ensures that all supplied information is correct.
    /// This method is provided for advanced usage only.
    pub fn remove_with_args(&self, kind: &Id, name: &Id, lang: u16) -> Result<&Self> {
        // if let Some(handle) = self.handle.lock().unwrap().as_ref() {
        if let Some(handle) = self.module_handle.lock().unwrap().as_ref() {
            let success = unsafe { UpdateResourceA(*handle, kind, name, lang, None, 0).as_bool() };

            if !success {
                return Err(format!(
                    "Resources::load(): Error removing resources: {:?}",
                    get_last_error()
                )
                .into());
            }
        } else {
            return Err(format!("resource '{}' is not open", self.file.to_str().unwrap()).into());
        };

        Ok(self)
    }

    /// Replace (Update) the resource in the resource file. It is expected that this is the
    /// original resource with the modified raw data.
    pub fn try_replace(&self, resource: &Resource) -> Result<&Self> {
        self.replace_with_args(
            &resource.kind,
            &resource.name,
            resource.lang,
            &resource.encoded.lock().unwrap(),
        )?;
        Ok(self)
    }

    /// Replace (Update) the resource in the resource file by supplying the resource type, name and lang
    /// as well as a `u8` slice containing the raw resource data.  Please note that if this function fails
    /// the entire resoruce update set may fail.
    pub fn replace_with_args(&self, kind: &Id, name: &Id, lang: u16, data: &[u8]) -> Result<&Self> {
        if let Some(handle) = self.module_handle.lock().unwrap().as_ref() {
            let success = unsafe {
                UpdateResourceA(
                    *handle,
                    kind,
                    name,
                    lang,
                    Some(std::mem::transmute(data.as_ptr())),
                    data.len() as u32,
                )
                .as_bool()
            };

            if !success {
                return Err(format!(
                    "Resources::load(): Error updating resources: {:?}",
                    get_last_error()
                )
                .into());
            }
        } else {
            return Err(format!(
                "resource file '{}' is not open",
                self.file.to_str().unwrap()
            )
            .into());
        };

        Ok(self)
    }

    /// Close the resource file.  This applies all the changes (updates) to the resource file.
    pub fn close(&mut self) {
        if let Some(handle) = self.module_handle.lock().unwrap().take() {
            unsafe {
                EndUpdateResourceA(handle, false);
            };
        }
    }

    /// Close the resource file discarding all changes.
    pub fn discard(&mut self) {
        if let Some(handle) = self.module_handle.lock().unwrap().take() {
            unsafe {
                EndUpdateResourceA(handle, true);
            };
        }
    }

    /// Create a new resource entry in the resource file. This function
    /// expects a valid [`Resource`] structure containing an appropriate
    /// resource type, name and raw data.
    pub fn insert(&self, r: Resource) {
        self.list.lock().unwrap().push(Arc::new(r))
    }

    /// Locate a resource entry by type and name.
    pub fn find(&self, typeid: Id, nameid: Id) -> Option<Arc<Resource>> {
        for item in self.list.lock().unwrap().iter() {
            if item.kind == typeid && item.name == nameid {
                return Some(item.clone());
            }
        }

        None
    }

    /// Locate and deserialize VS_VERSIONINFO structure (represented by [`VersionInfo`]).
    pub fn get_version_info(&self) -> Result<Option<VersionInfo>> {
        for item in self.list.lock().unwrap().iter() {
            if item.kind == resource_type::VERSION {
                return Ok(Some(item.clone().try_into()?));
            }
        }

        Ok(None)
    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        self.close();
    }
}

unsafe extern "system" fn enum_languages(
    hmodule: HINSTANCE,
    lptype: PCSTR,
    lpname: PCSTR,
    lang: u16,
    lparam: isize,
) -> BOOL {
    let rptr: *const Resources = std::mem::transmute(lparam);
    let hresinfo = match FindResourceExA(hmodule, lptype, lpname, lang) {
        Ok(hresinfo) => hresinfo,
        Err(e) => panic!("Unable to find resource {hmodule:?} {lptype:?} {lpname:?} {lang}: {e}"),
    };
    let resource = LoadResource(hmodule, hresinfo);
    let len = SizeofResource(hmodule, hresinfo);
    let data_ptr = LockResource(resource);
    let data = std::slice::from_raw_parts(std::mem::transmute(data_ptr), len as usize);
    let resources = &*rptr;
    resources.insert(Resource::new(resources, lptype, lpname, lang, data));
    BOOL(1)
}

unsafe extern "system" fn enum_names(
    hmodule: HINSTANCE,
    lptype: PCSTR,
    lpname: PCSTR,
    lparam: isize,
) -> BOOL {
    EnumResourceLanguagesA(hmodule, lptype, lpname, Some(enum_languages), lparam);
    BOOL(1)
}

unsafe extern "system" fn enum_types(hmodule: HINSTANCE, lptype: PCSTR, lparam: isize) -> BOOL {
    EnumResourceNamesA(hmodule, lptype, Some(enum_names), lparam);
    BOOL(1)
}
