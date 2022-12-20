//!
//! This crate allows you to load and modify Windows resources inside of `.exe` 
//! and `.res` files.  This crate currently does not support actual resource
//! data destructuring with exception of Version Strings (VS_VERSIONINFO), 
//! which is useful to modify application manifests. Loaded resources are 
//! available as raw `Vec<u8>` data, useful to modify bitmaps and icons.
//! 
//! ### Example
//! 
//! ```rust
//! let mut resources = Resources::new(&Path::new("myfile.exe"));
//! resources.load().expect("Unable to load resources");
//! resources.open().expect("Unable to open resource file for updates");
//! 
//! resources.find(resource_type::ICON,Id::Integer(1))
//!     .expect("unable to find main icon")
//!     .replace(icon_data)?
//!     .update()?;
//! 
//! let version: [u16;4] = [0,1,0,0];
//! resources.get_version_info()?.expect("Unable to get version info")
//!     .set_file_version(&version)
//!     .set_product_version(&version)
//!     .insert_strings(
//!         &[
//!             ("ProductName","My Product")
//!             ("FileDescription","My File")
//!         ]
//!     )
//!     .remove_string("SomeExistingString")
//!     .update()?;
//! 
//! resources.close();
//! ```
//! 
mod result;
mod error;
pub use error::*;
mod version;
pub use version::*;
mod id;
pub use id::*;
mod resources;
pub use resources::*;
mod utils;
