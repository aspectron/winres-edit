use std::collections::HashMap;
use crate::error::Error;
use crate::result::Result;
use manual_serializer::*;
use crate::utils::*;

#[derive(Debug)]
pub struct Header {
    pub length : usize,
    pub value_length: usize,
    pub data_type : DataType,
    pub key : String,
    pub last : usize,
}

impl Header {
    pub fn new(
        length : usize,
        value_length : usize,
        data_type : DataType,
        key : &str,
    ) -> Header {
        Header {
            length,
            value_length,
            data_type,
            key : key.to_string(),
            last : 0
        }
    }
}

impl TrySerialize for Header {
    type Error = Error;
    fn try_serialize(&self, dest: &mut Serializer) -> Result<()> {
        dest.try_align_u32()?;
        dest.try_u16(self.length as u16)?;
        dest.try_u16(self.value_length as u16)?;
        match self.data_type {
            DataType::Binary => {
                dest.try_u16(0)?;
            },
            DataType::Text => {
                dest.try_u16(1)?;
            }
        };
        dest.try_utf16sz(&self.key)?;
        dest.try_align_u32()?;
        Ok(())
    }
}

impl TryDeserialize for Header {
    type Error = Error;

    fn try_deserialize(src: &mut Deserializer) -> Result<Header> {
        src.try_align_u32()?;

        let cursor = src.cursor();
        let length = src.try_u16()? as usize;
        let value_length = src.try_u16()? as usize;
        let data_type = src.try_u16()?;
        println!("@ cursor: {cursor} length: {length} value_length: {value_length} data_type: {data_type}");
        let data_type = match data_type {
            0 => DataType::Binary,
            1 => DataType::Text,
            _ => return Err(format!("invalid version resource data type").into())
        };
        let key = src.try_utf16sz()?;

        let padding = src.cursor() % 4;
        println!("$---: padding: {padding}");
        src.try_offset(padding)?;
        let last = cursor + length;

        let header = Header {length,value_length,data_type,key,last};
        println!("{:#?}", header);
        Ok(header)
    }
}


pub fn try_build_struct(
    key : &str,
    data_type : DataType,
    value_len: usize,
    value : &[u8]
) -> Result<Vec<u8>> {
    let mut dest = Serializer::new(4096);
    let header = Header::new(0,0,data_type,key);
    // println!("header {:?}",header);
    dest.try_serialize(&header)?;
    dest.try_u8slice(value)?;
    let mut vec = dest.to_vec();
    store_u16(&mut vec[0..2], dest.len() as u16);
    store_u16(&mut vec[2..4], value_len as u16);
    // println!("vec: {:?}", vec);
    Ok(vec)
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub signature : u32,
    pub struc_version : u32,
    pub file_version_ms : u32,
    pub file_version_ls : u32,
    pub product_version_ms : u32,
    pub product_version_ls : u32,
    pub file_flags_mask : u32,
    pub file_flags : u32,
    pub file_os : u32,
    pub file_type : u32,
    pub file_subtype : u32,
    pub file_date_ms : u32,
    pub file_date_ls : u32,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            signature: 0xfeef04bd,
            struc_version: 0,
            file_version_ms: 0,
            file_version_ls: 0,
            product_version_ms: 0,
            product_version_ls: 0,
            file_flags_mask: 0,
            file_flags: 0,
            file_os: 0,
            file_type: 0,
            file_subtype: 0,
            file_date_ms: 0,
            file_date_ls: 0,
        }
    }
}

impl FileInfo {
    pub fn print(&self) {
        println!("signature: 0x{:x}", self.signature);
        println!("struc_version: 0x{:x}", self.struc_version);
        println!("file_version_ms: {} (0x{:x})", self.file_version_ms, self.file_version_ms);
        println!("file_version_ls: {} (0x{:x})", self.file_version_ls, self.file_version_ls);
        println!("file_version: {:?}", self.get_file_version());
        println!("product_version_ms: {} (0x{:x})", self.product_version_ms, self.product_version_ms);
        println!("product_version_ls: {} (0x{:x})", self.product_version_ls, self.product_version_ls);
        println!("product_version: {:?}", self.get_product_version());
        println!("file_flags_mask: 0x{:x}", self.file_flags_mask);
        println!("file_flags: 0x{:x}", self.file_flags);
        println!("file_os: 0x{:x}", self.file_os);
        println!("file_type: 0x{:x}", self.file_type);
        println!("file_subtype: 0x{:x}", self.file_subtype);
        println!("file_date_ms: {}", self.file_date_ms);
        println!("file_date_ls: {}", self.file_date_ls);
    }

    // pub fn test(&mut self) {
    //     let v = vec![1,2,3,4];
    //     self.set_file_version(v.as_slice().try_into().unwrap());
    // }

    pub fn get_file_version(&self) -> [u16;4] {
        u32msls_as_u16vec(self.file_version_ms, self.file_version_ls)
    }

    pub fn set_file_version(&mut self, v: &[u16;4]) {
        u16vec_to_u32msls(v,&mut self.file_version_ms, &mut self.file_version_ls);
    }

    pub fn get_product_version(&self) -> [u16;4] {
        u32msls_as_u16vec(self.product_version_ms, self.product_version_ls)
    }
    
    pub fn set_product_version(&mut self, v: &[u16;4]) {
        u16vec_to_u32msls(v,&mut self.product_version_ms, &mut self.product_version_ls);
    }

}
// impl TryFrom<&mut Deserializer<'_>> for FileInfo {
impl TryDeserialize for FileInfo {
    type Error = Error;
    fn try_deserialize(src: &mut Deserializer) -> Result<FileInfo> {
        // let src = Deserializer::new(data);

        let info = FileInfo {
            signature : src.try_u32()?,
            struc_version : src.try_u32()?,
            file_version_ms : src.try_u32()?,
            file_version_ls : src.try_u32()?,
            product_version_ms : src.try_u32()?,
            product_version_ls : src.try_u32()?,
            file_flags_mask : src.try_u32()?,
            file_flags : src.try_u32()?,
            file_os : src.try_u32()?,
            file_type : src.try_u32()?,
            file_subtype : src.try_u32()?,
            file_date_ms : src.try_u32()?,
            file_date_ls : src.try_u32()?,
        };

        if info.signature != 0xfeef04bd {
            return Err(format!("FileInfo: invalid signature 0x{:8x}", info.signature).into());
        }

        Ok(info)
    }
}

impl TrySerialize for FileInfo {
    type Error = Error;
    fn try_serialize(&self, dest: &mut Serializer) -> Result<()> {
        // let mut dest = Serializer::new(std::mem::size_of::<FileInfo>());
        dest
            .try_u32(self.signature)?
            .try_u32(self.struc_version)?
            .try_u32(self.file_version_ms)?
            .try_u32(self.file_version_ls)?
            .try_u32(self.product_version_ms)?
            .try_u32(self.product_version_ls)?
            .try_u32(self.file_flags_mask)?
            .try_u32(self.file_flags)?
            .try_u32(self.file_os)?
            .try_u32(self.file_type)?
            .try_u32(self.file_subtype)?
            .try_u32(self.file_date_ms)?
            .try_u32(self.file_date_ls)?;

        Ok(())
            // .to_vec()
    }
}

#[derive(Debug, Clone)]
pub enum VersionInfoChild {
    StringFileInfo {
        tables : HashMap<String, HashMap<String,Data>>
    },
    VarFileInfo {
        vars : HashMap<String,Vec<u32>>    
    },
}

#[derive(Debug, Clone)]
pub enum Data {
    Binary(Vec<u8>),
    Text(String)
}

impl TrySerialize for VersionInfoChild {
    type Error = Error;
    fn try_serialize(&self, dest: &mut Serializer) -> Result<()> {

        match self {
            VersionInfoChild::StringFileInfo { tables } => {

                for (key_lang,map) in tables {

                    let mut lang_records = Serializer::default();

                    for (key_record, data) in map {

                        let (data_type,data) = match data {
                            Data::Binary(data) => {
                                (
                                    DataType::Binary,
                                    data.clone()
                                )
                            },
                            Data::Text(text) => {
                                (
                                    DataType::Text,
                                    utf16sz_to_u8vec(text)
                                )
                            },
                        };

                        let string_record = try_build_struct(key_record,data_type,data.len()/2,&data)?;
                        lang_records.try_align_u32()?;
                        lang_records.try_u8slice(&string_record)?;
                    }
                    
                    let string_table = try_build_struct(key_lang,DataType::Binary,0,&lang_records.to_vec())?;
                    let string_file_info = try_build_struct("StringFileInfo",DataType::Binary,0,&string_table)?;
                    dest.try_align_u32()?;
                    dest.try_u8slice(&string_file_info)?;
                }
            },
            VersionInfoChild::VarFileInfo { vars } => {
                let mut var_records = Serializer::default();
                for (k,data) in vars {
                    let var_record = try_build_struct(k,DataType::Binary,data.len()/2,&u32slice_to_u8vec(data))?;
                    var_records.try_align_u32()?;
                    var_records.try_u8slice(&var_record)?;
                }
                let var_file_info = try_build_struct("VarFileInfo",DataType::Binary,0,&var_records.to_vec())?;
                dest.try_align_u32()?;
                dest.try_u8slice(&var_file_info)?;
            }
        }

        Ok(())
    }
}

impl TryDeserialize for VersionInfoChild {
    type Error = Error;
    fn try_deserialize(src: &mut Deserializer) -> Result<VersionInfoChild> {

        let header: Header = src.try_deserialize()?;

        let data = match header.key.as_str() {
            "StringFileInfo" => {
                let mut tables = HashMap::new();
                while src.cursor() < header.last {

                    println!("loading string table");
                    let string_table_header: Header = src.try_deserialize()?;
                    let lang = string_table_header.key;
                    let mut data = HashMap::new();
        
                    while src.cursor() < string_table_header.last {
                        println!("loading string table record");
                        let string_header: Header = src.try_deserialize()?;
                        match string_header.data_type {
                            DataType::Binary => {
                                println!("!!! BINARY DATA !!!");
                                let len = string_header.value_length*2;
                                let vec = src.try_u8vec(len)?;
                                data.insert(string_header.key, Data::Binary(vec));
                            },
                            DataType::Text => {
                                let text = src.try_utf16sz()?;
                                data.insert(string_header.key, Data::Text(text));
                            }
                        };
                    }
        
                    tables.insert(lang, data);
                }
    
                VersionInfoChild::StringFileInfo { tables }
            },
            "VarFileInfo" => {

                let mut vars = HashMap::new();
                while src.cursor() < header.last {
                    // let var_header = Header::try_from(&mut *src)?;
                    let var_header: Header = src.try_deserialize()?;
        
                    let mut values = Vec::new();
                    while src.cursor() < var_header.last {
                        values.push(src.try_u32()?);
                    }
        
                    vars.insert(var_header.key, values);
                }

                VersionInfoChild::VarFileInfo { 
                    vars
                }
            },
            _ => return Err(format!("Unknown child type: {}", header.key).into())
        };

        Ok(data)

    }

}

#[derive(Debug, Clone)]
pub enum DataType {
    Binary,
    Text,
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub data_type : DataType,
    pub key : String,
    pub info : FileInfo,
    pub children : Vec<VersionInfoChild>,
}

impl TryFrom<&[u8]> for VersionInfo {
    type Error = Error;
    fn try_from(data: &[u8]) -> Result<VersionInfo> {
        let mut src = Deserializer::new(data);
        println!("#----- remaining at start: {}", src.remaining());
        
        let header: Header = src.try_deserialize()?;
        println!("#----- remaining after VSI header: {}", src.remaining());
        println!("HEADER: {:#?}", header);
        println!("FileInfoHeader size: {}", std::mem::size_of::<FileInfo>());
        let info :FileInfo = src.try_deserialize()?;
        let skip = src.cursor() % 4;
        src.try_offset(skip)?;
        println!("skip: {}", skip);
        println!("#----- remaining after FileInfo: {}", src.remaining());

        let mut children = Vec::new();
        let mut remaining = src.remaining();
        while remaining > 0 {
            println!("#----- remaining before: {}", src.remaining());
            
            let child: VersionInfoChild = src.try_deserialize()?;
            children.push(child);
            remaining = src.remaining();
            println!("#----- remaining before: {}", remaining);
        }

        let info = VersionInfo {
            data_type : header.data_type,
            key : header.key,
            info,
            children
        };

        Ok(info)

    }
}

impl VersionInfo {
    pub fn try_to_vec(&self) -> Result<Vec<u8>> {
        let mut dest = Serializer::default();

        let mut child_data = Serializer::default();
        for child in &self.children {
            child_data.try_serialize(child)?;
            child_data.try_align_u32()?;
        }
        let child_data = child_data.to_vec();

        let file_info_data = Serializer::default()
            .try_serialize(&self.info)?
            .to_vec();

        let data = Serializer::default()
            .try_u8slice(&file_info_data)?
            .try_align_u32()?
            .try_u8slice(&child_data)?
            .to_vec();

        let version_info = try_build_struct("VS_VERSION_INFO",DataType::Binary,file_info_data.len(),&data)?;
        dest.try_u8slice(&version_info)?;
        // dest.try_align_u32()?;
        // dest.try_u8slice(&child_data)?;

        Ok(dest.to_vec())
    }

    pub fn set_file_version(&mut self, v : &[u16;4]) {
        self.info.set_file_version(v);
        self.insert_string("FileVersion", &format_version_string(v))
    }
    
    pub fn set_product_version(&mut self, v : &[u16;4]) {
        self.info.set_product_version(v);
        self.insert_string("ProductVersion", &format_version_string(v))
    }

    pub fn set_version(&mut self, v: &[u16;4]) {
        self.set_file_version(v);
        self.set_product_version(v);
    }

    pub fn replace_string(&mut self, key: &str, text: &str) {
        for child in self.children.iter() {
            match child {
                VersionInfoChild::StringFileInfo { tables } => {
                    for (lang, table) in tables {
                        if let Some(text) = table.get(key) {
                            table.insert(key.to_string(), Data::Text(text.to_string()));
                        }
                    }
                },
                _ => { }
            }
        }
    }

    pub fn insert_string(&mut self, key: &str, text: &str) {
        for child in self.children.iter() {
            match child {
                VersionInfoChild::StringFileInfo { tables } => {
                    for (lang, table) in tables {
                        table.insert(key.to_string(), Data::Text(text.to_string()));
                    }
                },
                _ => { }
            }
        }
    }

    pub fn remove_string(&mut self, key: &str, text: &str) {
        for child in self.children.iter() {
            match child {
                VersionInfoChild::StringFileInfo { tables } => {
                    for (lang, table) in tables {
                        table.remove(key);
                    }
                },
                _ => { }
            }
        }
    }

    pub fn ensure_language(&mut self, lang: &str) {
        for child in self.children.iter() {
            match child {
                VersionInfoChild::StringFileInfo { tables } => {
                    if tables.get(lang).is_none() {
                        tables.insert(lang.to_string(), HashMap::new());
                    }
                },
                _ => { }
            }
        }
    }
    
}
