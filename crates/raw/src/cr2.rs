use std::fs::File;
use std::io::{self,Seek,Read};
use std::str;
use std::mem;
use std::fmt;
use std::error::Error;
use std::collections::HashMap;
use std::any::Any;
use std::ops::Deref;

    enum ByteOrder {
        Intel,
        Motorola
    }

    impl Default for ByteOrder{
        fn default() -> ByteOrder { ByteOrder::Intel }
    }
    
    pub enum RawFileError {
        Io(io::Error),
        Utf8(str::Utf8Error),
        FileFormat(String),
        Seek(u64),
        NotImplemented(String),
        TypeError(u16)
    }

    impl fmt::Display for RawFileError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RawFileError::Io(ref e) => {write!(f,"IO error: {}",e.description())},
            RawFileError::Utf8(ref e) => {write!(f,"Utf8 conversion error: {}",e.description())},
            RawFileError::FileFormat(ref s) => {write!(f,"File format error: {}",s)},
            RawFileError::Seek(p) => {write!(f,"Seek error: {}",p)},
            RawFileError::NotImplemented(ref s) => {write!(f,"Feature not Implemented: {}",s)}
            RawFileError::TypeError(u) => {write!(f,"Unknown Type: {}",u)}
        }} 
    }

    impl From<io::Error> for RawFileError {
        fn from(e: io::Error) -> RawFileError {
            RawFileError::Io(e)
        }
    }

    impl From<str::Utf8Error> for RawFileError {
        fn from(e: str::Utf8Error) -> RawFileError {
            RawFileError::Utf8(e)
        }
    }

    enum TagData {
        Unsigned(u32),
        Signed(i32),
        U64(u64),
        I64(i64),
        Strg(String),
        Float(f64)
    }

    #[derive(Default)]
    pub struct RawImage<'a> {
        pub file_name:  Box<String>,
        byte_order: ByteOrder,
        pub raw_offset: u32,
        ifd_offsets: Vec<u32>,
        tags: HashMap<String,&'a Any>
    }


pub fn open<'a>(path: String) -> Result<RawImage<'a>,RawFileError>{

    let mut f = try!(File::open(&path));
    let mut ri: RawImage = Default::default();
    try!(ri.read_header(&mut f));
    let mut i=0;
    while ri.ifd_offsets.len() > i {
        try!(ri.read_ifd(&mut f,i,true));
        i += 1;
    }
    ri.file_name = Box::new(String::from(path));
    Ok(ri)
}

trait FloatConversion {
    fn to_f32(&self) -> Option<f32>;
    fn to_f64(&self) -> Option<f64>;
}

impl FloatConversion for [u8] {
    fn to_f32(&self) -> Option<f32> {
        if self.len() == 4 {
            let mut val = [0u8;4];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;4],f32>(val)});
        }
        None
    }

    fn to_f64(&self) -> Option<f64> {
        if self.len() == 8 {
            let mut val = [0u8;8];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;8],f64>(val)});
        }
        None
    }


}

trait IntConversion {
    fn to_u8(&self) -> Option<u8>;
    fn to_u16(&self) -> Option<u16>;
    fn to_u32(&self) -> Option<u32>;
    fn to_i8(&self) -> Option<i8>;
    fn to_i16(&self) -> Option<i16>;
    fn to_i32(&self) -> Option<i32>;
    fn to_u64(&self) -> Option<u64>;
    fn to_i64(&self) -> Option<i64>;
 
}

impl IntConversion for [u8] {
    fn to_u8(&self) -> Option<u8> {
        if self.len() == 1 {
            return Some(self[0]);
        }
        None
    }
    fn to_u16(&self) -> Option<u16> {
        if self.len() == 2 {
            let mut val = [0u8;2];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;2],u16>(val)});
        }
        None
    }

    fn to_u32(&self) -> Option<u32> {
        if self.len() == 4 {
            let mut val = [0u8;4];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;4],u32>(val)});
        }
        None
    }

    fn to_i8(&self) -> Option<i8> {
        if self.len() == 1 {
            let mut val = [0u8;1];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;1],i8>(val)});
        }
        None
    }

    fn to_i16(&self) -> Option<i16> {
        if self.len() == 2 {
            let mut val = [0u8;2];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;2],i16>(val)});
        }
        None
    }

    fn to_i32(&self) -> Option<i32> {
        if self.len() == 4 {
            let mut val = [0u8;4];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;4],i32>(val)});
        }
        None
    }

    fn to_u64(&self) -> Option<u64> {
        if self.len() == 8 {
            let mut val = [0u8;8];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;8],u64>(val)});
        }
        None
    }

    fn to_i64(&self) -> Option<i64> {
        if self.len() == 8 {
            let mut val = [0u8;8];
            val.clone_from_slice(self);
            return Some(unsafe{ mem::transmute::<[u8;8],i64>(val)});
        }
        None
    }

}


impl<'a> RawImage<'a> {
fn read_header(&mut self,f: &mut File) -> Result<(),RawFileError> {
    
    if 0 != try!(f.seek(::std::io::SeekFrom::Start(0))) { return Err(RawFileError::Seek(0)) } ;
    let mut head = [0u8; 16];
    try!(f.read(&mut head));
    
    let bo = &head[0..2]; // Byte order
    let s = try!(str::from_utf8(&bo));
    match s {
        "II" => self.byte_order = ByteOrder::Intel,
        "MM" => self.byte_order = ByteOrder::Motorola,
        _    => return Err(RawFileError::FileFormat("Unknown byte order ".to_string()+s)) 
    }
    if s != "II" { return Err(RawFileError::NotImplemented("Only Intel Byte Order supported!".to_string())) };
    
    if head[2..4].to_u16().unwrap() != 0x002a { return Err(RawFileError::FileFormat("Tiff Magic mismatch".to_string()))};
        
    let mut to = [ 0u8; 4];        // Tiff Offset
    to.clone_from_slice(&head[4..8]);
    self.ifd_offsets.push(head[4..8].to_u32().unwrap());
    
    let cm = &head[8..10];         // CR2 Magic
    if try!(str::from_utf8(&cm)) != "CR" { return Err(RawFileError::FileFormat("CR2 Magic mismatch".to_string()));}
    
    let cmaj = &head[10..11];        // CR2 Major
    let cmin = &head[11..12];        // CR2 Minor
    if cmaj[0]!=2 && cmin[0]!=0 { return Err(RawFileError::NotImplemented(format!("CR2 Version {}.{} not supported",cmaj[0],cmin[0])));}
    
    self.raw_offset = head[12..16].to_u32().unwrap();
    Ok(())

}

fn read_tag(&mut self, f: &mut File) -> Result<(),RawFileError>{
    let mut tag = [0u8; 12];
    try!(f.read(&mut tag));
    let tagid = tag[0..2].to_u16().unwrap();
    let tagtype = tag[2..4].to_u16().unwrap();
    let valcount = tag[4..8].to_u32().unwrap() as usize; 
    let mut data: Vec<u8> = From::from(&tag[8..12]);
    let tagname = match tagid {
        0x103 => "compression",
        0x111 => "strip_offset",
        0x117 => "strip_byte_count",
        0xc640 => "strip_cr2_slice",
        _ => "???"
    };
    let valsize: usize = match tagtype {
        1|2|6|7 => 1,
        3|8 => 2,
        4|9|11 => 4,
        5|10|12 => 8,
        _ => 0
    };
    //println!("ID: {:0>4x}, type: {:2}, count: {:8x}, data: {:8x} {}",tagid,tagtype,valcount,tagdata,tagname);
    if valsize*valcount > 4
    {
        let offset = tag[8..12].to_u32().unwrap();
        let mut f = try!(File::open(self.file_name.deref()));
        try!(f.seek(io::SeekFrom::Start(offset as u64)));
        data = vec![0u8; (valsize * valcount) as usize];
        try!(f.read(&mut data));
    }
        let mut d : Vec<TagData> = Vec::new();
        let mut s:  String = String::new(); 
        for w in data.windows(valsize) {
            match tagtype {
                1|7 => d.push(TagData::Unsigned(w.to_u8().unwrap() as u32)),
                2 => s.push(w.to_u8().unwrap() as char),
                3 => d.push(TagData::Unsigned(w.to_u16().unwrap() as u32)),
                4 => d.push(TagData::Unsigned(w.to_u32().unwrap())),
                5 => d.push(TagData::U64(w.to_u64().unwrap())),
                6 => d.push(TagData::Signed(w.to_i8().unwrap() as i32)),
                8 => d.push(TagData::Signed(w.to_i16().unwrap() as i32)),
                9 => d.push(TagData::Signed(w.to_i32().unwrap())),
                10 => d.push(TagData::I64(w.to_i64().unwrap())),
                11 => d.push(TagData::Float(w.to_f32().unwrap() as f64)),
                12 => d.push(TagData::Float(w.to_f64().unwrap())),
                _ => return Err(RawFileError::TypeError(tagtype))

            }    
        }
    
    Ok(())
}

fn read_ifd(&mut self,f: &mut File, index: usize,read_tags:bool) -> Result<u32,RawFileError>{
    let mut pos = try!(f.seek(io::SeekFrom::Start(self.ifd_offsets[index] as u64)));
    let mut na=[0u8; 2];
    try!(f.read(&mut na));
    let n = na.to_u16().unwrap();
    if read_tags {
        for n in 0..n {
            self.read_tag(f);
        }
    }
    pos=pos+n as u64 *12+2;
    let mut ioa = [0u8; 4];
    try!(f.seek(io::SeekFrom::Start(pos)));
    try!(f.read(&mut ioa));
    let io = ioa.to_u32().unwrap();
    if io != 0 {self.ifd_offsets.push(io)};
    Ok(io)

}

}


