use std::collections::HashMap;
use crate::error::Error;
use crate::result::Result;
use manual_serializer::*;


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
            signature: 0,
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

        if info.signature != 0xFEEF04BD {
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
    u16vec.resize(len,0);
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
// pub fn u8slice_to_u16vec(data: &[u8]) -> Vec<u16> {
//     let len = data.len();
//     let words = len / 2 + (len % 2);
//     let mut vec = Vec::with_capacity(words);
//     vec.resize(words,0);
//     let src = unsafe { std::mem::transmute(vec.as_ptr()) };
//     let dest = vec[0..].as_mut_ptr();
//     unsafe { std::ptr::copy(src,dest,len); }
//     vec
// }

pub fn try_build_struct(
    key : &str,
    data_type : DataType,
    value_len: usize,
    value : &[u8]
) -> Result<Vec<u8>> {
    let mut dest = Serializer::new(4096);

    let header = Header::new(0,0,data_type,key);
    // let value_len = value.len();

    dest.try_serialize(&header)?;
    dest.try_u8slice(value)?;

    let mut vec = dest.to_vec();
    store_u16(&mut vec[0..2], dest.len() as u16);
    store_u16(&mut vec[2..4], value_len as u16);

    Ok(vec)
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
                                    DataType::Binary,
                                    utf16sz_to_u8vec(text)
                                )
                            },
                        };

                        let string_record = try_build_struct(key_record,data_type,data.len(),&data)?;
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

        let file_info_data = Serializer::default().try_serialize(&self.info)?.to_vec();

        let version_info = try_build_struct("VS_VERSION_INFO",DataType::Binary,file_info_data.len(),&file_info_data)?;
        dest.try_u8slice(&version_info)?;
        dest.try_align_u32()?;
        dest.try_u8slice(&child_data)?;

        Ok(dest.to_vec())
    }
}
