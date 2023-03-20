/*!
 [<img alt="github" src="https://img.shields.io/badge/github-aspectron/winres--edit-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/aspectron/winres-edit)
[<img alt="crates.io" src="https://img.shields.io/crates/v/winres-edit.svg?maxAge=2592000&style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/winres-edit)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-winres--edit-56c2a5?maxAge=2592000&style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/winres-edit)
<img alt="license" src="https://img.shields.io/crates/l/winres-edit.svg?maxAge=2592000&color=6ac&style=for-the-badge&logoColor=fff" height="20">

This crate allows you to load and modify Windows resources inside of `.exe`
and `.res` files.  This crate currently does not support actual resource
data destructuring with exception of Version Strings (VS_VERSIONINFO),
which is useful to modify application manifests. Loaded resources are
available as raw `Vec<u8>` data, useful to modify bitmaps and icons.

Please note that all operations performed on the opened resource file are accumulated and are then "flushed" to the file when the file is closed
using the `close()` function. This is due to the behavior of the underlying Win32 API ([UpdateResource](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-updateresourcea)) functionality used by this crate.

### Example

#### Load and update a resource
```rust
let mut resources = Resources::new(&Path::new("myfile.exe"));
resources.load().expect("Unable to load resources");
resources.open().expect("Unable to open resource file for updates");

resources.find(resource_type::ICON,Id::Integer(1))
    .expect("unable to find main icon")
    .replace(icon_data)?
    .update()?;

let version: [u16;4] = [0,1,0,0];
resources.get_version_info()?.expect("Unable to get version info")
    .set_file_version(&version)
    .set_product_version(&version)
    .insert_strings(
        &[
            ("ProductName","My Product")
            ("FileDescription","My File")
        ]
    )
    .remove_string("SomeExistingString")
    .update()?;

resources.close();
```

#### Create a new resource

```rust
let res = Resource::new(
    &resources,
    resource_type::ICON.into(),
    Id::Integer(14).into(),
    1033,
    target_icon.data(),
);
res.update()?;
*/

mod error;
mod id;
mod resources;
mod result;
mod utils;
mod version;

pub use error::*;
pub use id::*;
pub use resources::*;
pub use version::*;
