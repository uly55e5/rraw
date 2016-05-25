use std::fs::File;
use std::io::{self,Seek,Read};
use std::str;
use std::mem;
use std::fmt;
use std::error::Error;
use std::collections::HashMap;
use std::any::Any;
use std::ops::Deref;

/// Byte order of the containing data
enum ByteOrder {
    /// little endian
    Intel,
    /// big endian
    Motorola
}

/// Sets the default byte order to little endian
/// TODO big endian is not implemented yet
impl Default for ByteOrder{
    fn default() -> ByteOrder { ByteOrder::Intel }
}

/// Error types for the raw file reader
pub enum RawFileError {
    Io(io::Error),
    Utf8(str::Utf8Error),
    FileFormat(String),
    Seek(u64),
    NotImplemented(String),
    TypeError(u16)
}

/// Display error messages
impl fmt::Display for RawFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         match *self {
            RawFileError::Io(ref e) => {write!(f,"IO error: {}",e.description())},
            RawFileError::Utf8(ref e) => {write!(f,"Utf8 conversion error: {}",e.description())},
            RawFileError::FileFormat(ref s) => {write!(f,"File format error: {}",s)},
            RawFileError::Seek(p) => {write!(f,"Seek error: {}",p)},
            RawFileError::NotImplemented(ref s) => {write!(f,"Feature not Implemented: {}",s)}
            RawFileError::TypeError(u) => {write!(f,"Unknown Type: {}",u)}
        }
    } 
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

    #[derive(Debug)]
    enum TagData {
        Unsigned(u32),
        Signed(i32),
        U64(u64),
        I64(i64),
        Strg(String),
        Float(f64)
    }

    struct Ifd {
        offset: usize,
        tags: HashMap<String, Vec<TagData>>
            
    
    }

    #[derive(Default)]
    pub struct RawImage {
        pub file_name:  Box<String>,
        byte_order: ByteOrder,
        pub raw_offset: usize,
        ifd: Vec<Ifd>,
        tags: HashMap<String,Vec<TagData> >
    }


pub fn open(path: String) -> Result<RawImage,RawFileError>{

    let mut file = try!(File::open(&path));
    let mut image: RawImage = Default::default();
    image.file_name = Box::new(String::from(path));
    try!(image.read_header(&mut file));
    let mut i=0;
    while image.ifd.len() > i {
        try!(image.read_ifd(&mut file,i,true));
        i += 1;
    }
    Ok(image)
}

trait Transmute {
    fn to<T:Copy>(&self) -> Option<T>;
}

impl Transmute for [u8] {
    fn to<T: Copy>(&self) -> Option<T> {
        let tlen: usize = mem::size_of::<T>();
        if self.len() == tlen
        {
            let val = self.as_ptr()  as *const T;;
            return Some(unsafe{(*val)});
        }
        None
    }
}


impl<'a> RawImage {
    fn read_header(&mut self,f: &mut File) -> Result<(),RawFileError> {
        if 0 != try!(f.seek(::std::io::SeekFrom::Start(0))) { 
            return Err(RawFileError::Seek(0)) 
        } ;
        let mut head = [0u8; 16];
        try!(f.read(&mut head));
    
        let bo = &head[0..2]; // Byte order
        let s = try!(str::from_utf8(&bo));
        match s {
            "II" => self.byte_order = ByteOrder::Intel,
            "MM" => self.byte_order = ByteOrder::Motorola,
            _    => return Err(RawFileError::FileFormat("Unknown byte order ".to_string()+s)) 
        }
        if s != "II" { 
            return Err(RawFileError::NotImplemented("Only Intel Byte Order supported!".to_string())) 
        };
    
        if head[2..4].to::<u16>().unwrap() != 0x002a { 
            return Err(RawFileError::FileFormat("Tiff Magic mismatch".to_string()))
        };
        
        let mut to = [ 0u8; 4];        // Tiff Offset
        to.clone_from_slice(&head[4..8]);
        self.ifd.push(Ifd{offset: head[4..8].to::<u32>().unwrap() as usize,tags: HashMap::new()});
    
        let cm = &head[8..10];         // CR2 Magic
        if try!(str::from_utf8(&cm)) != "CR" { 
            return Err(RawFileError::FileFormat("CR2 Magic mismatch".to_string()));
        }
    
        let cmaj = &head[10..11];        // CR2 Major
        let cmin = &head[11..12];        // CR2 Minor
        if cmaj[0]!=2 && cmin[0]!=0 {
            return Err(RawFileError::NotImplemented(format!(
                        "CR2 Version {}.{} not supported",cmaj[0],cmin[0])));
        }
    
        self.raw_offset = head[12..16].to::<u32>().unwrap() as usize;
        Ok(())
    }

    fn read_tag(&mut self, f: &mut File) -> Result<(),RawFileError>{
        let mut tag = [0u8; 12];
        try!(f.read(&mut tag));
        let tagid = tag[0..2].to::<u16>().unwrap();
        let tagtype = tag[2..4].to::<u16>().unwrap();
        let valcount = tag[4..8].to::<u32>().unwrap() as usize; 
        let mut data: Vec<u8> = From::from(&tag[8..12]);
        let tagname = match tagid {
            0x100 => "width",
            0x101 => "height",
            0x102 => "bits_per_sample",
            0x103 => "compression",
            0x10f => "make",
            0x110 => "model",
            0x111 => "strip_offset",
            0x112 => "orientation",
            0x117 => "strip_byte_count",
            0x11a => "x_resolution",
            0x11b => "y_resolution",
            0x128 => "res_unit",
            0x132 => "date_time",
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
        if valsize*valcount > 4
        {   
            let offset = tag[8..12].to::<u32>().unwrap();
            let mut f = try!(File::open(self.file_name.deref()));
            try!(f.seek(io::SeekFrom::Start(offset as u64)));
            data = vec![0u8; (valsize * valcount) as usize];
            try!(f.read(&mut data));
        }
        let mut d : Vec<TagData> = Vec::new();
        let mut s:  String = String::new(); 
        let mut i = 0;
        for w in data.chunks(valsize) {
            i =  i+1;;
            if i > valcount { 
                break; 
            }
            match tagtype {
                1|7 => d.push(TagData::Unsigned(w.to::<u8>().unwrap() as u32)),
                2 => s.push(w.to::<u8>().unwrap() as char),
                3 => d.push(TagData::Unsigned(w.to::<u16>().unwrap() as u32)),
                4 => d.push(TagData::Unsigned(w.to::<u32>().unwrap())),
                5 => d.push(TagData::U64(w.to::<u64>().unwrap())),
                6 => d.push(TagData::Signed(w.to::<i8>().unwrap() as i32)),
                8 => d.push(TagData::Signed(w.to::<i16>().unwrap() as i32)),
                9 => d.push(TagData::Signed(w.to::<i32>().unwrap())),
                10 => d.push(TagData::I64(w.to::<i64>().unwrap())),
                11 => d.push(TagData::Float(w.to::<f32>().unwrap() as f64)),
                12 => d.push(TagData::Float(w.to::<f64>().unwrap())),
                _ => return Err(RawFileError::TypeError(tagtype))
            }    
        }
        println!("name: {:20} id: {:0>4x}",tagname,tagid);
        Ok(())
    }

fn read_ifd(&mut self,f: &mut File, index: usize,read_tags:bool) -> Result<usize,RawFileError>{
    let mut pos = try!(f.seek(io::SeekFrom::Start(self.ifd[index].offset as u64)));
    let mut na=[0u8; 2];
    try!(f.read(&mut na));
    let n = na.to::<u16>().unwrap();
    if read_tags {
        for n in 0..n {
            let r = self.read_tag(f);
        }
    }
    pos=pos+n as u64 *12+2;
    let mut ioa = [0u8; 4];
    try!(f.seek(io::SeekFrom::Start(pos)));
    try!(f.read(&mut ioa));
    let io = ioa.to::<u32>().unwrap() as usize;
    if io != 0 {
        self.ifd.push(Ifd{offset: io,tags: HashMap::new()})
    }
    Ok(io)

}

}

#[test]
fn test_u8_array_to_int() {
    let a = [2u8; 10];

    assert_eq!(0x02,a[0..1].to::<u8>().unwrap());
    assert_eq!(0x0202,a[0..2].to::<u16>().unwrap());
    assert_eq!(0x02020202,a[0..4].to::<u32>().unwrap());
    assert_eq!(0x0202020202020202,a[0..8].to::<u64>().unwrap());
    assert_eq!(0x02,a[0..1].to::<i8>().unwrap());
    assert_eq!(0x0202,a[0..2].to::<i16>().unwrap());
    assert_eq!(0x02020202,a[0..4].to::<i32>().unwrap());
    assert_eq!(0x0202020202020202,a[0..8].to::<i64>().unwrap());
}

