use std::collections::HashMap;
use crate::error::Error;
use crate::result::Result;
use manual_serializer::*;

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
        key : String,
    ) -> Header {
        Header {
            length,
            value_length,
            data_type,
            key,
            last : 0
        }
    }
}



impl TrySerialize for Header {
    type Error = Error;
    fn try_serialize(&self, dest: &mut Serializer) -> Result<()> {

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
