# `winres-edit`

Crate for modification of windows resources.

[![Crates.io](https://img.shields.io/crates/l/winres-edit.svg?maxAge=2592000)](https://crates.io/crates/winres-edit)
[![Crates.io](https://img.shields.io/crates/v/winres-edit.svg?maxAge=2592000)](https://crates.io/crates/winres-edit)

### Overview

This crate allows you to load and modify Windows resources inside of `.exe` and `.res` files.  This crate currently does not support actual resource data destructuring with exception of Version Strings (VS_VERSION_INFO), which is useful to modify application manifests. Loaded resources are available as raw `Vec<u8>` data, useful to modify bitmaps and icons.

### Example

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

### Icons

This crate works well in conjunction with the [`ico`](https://crates.io/crates/ico) crate that can be used to load/store external `.ico` files as well as load `.png` files and encode them into windows-compatible resource format.

### Example

```rust
let iconfile = std::fs::File::open("myicon.ico").unwrap();
let icon_dir = ico::IconDir::read(iconfile).unwrap();    
let target_icon = icon_dir
    .entries()
    .iter()
    .find(|&e| e.width() == 256)
    .expect("can't find 256x256 icon");
let icon_data = target_icon.data();
```

This crate also works well in conjunction with the [`image`](https://crates.io/image) crate that can interact with the [`ico`](https://crates.io/crates/ico) crate to load, resize and and store custom icons within resource files.


