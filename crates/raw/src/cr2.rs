use std::fs::File;
use std::io::{self,Seek,Read};
use std::str;
use std::mem;
use std::fmt;
use std::error::Error;

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
        NotImplemented(String)
    }

    impl fmt::Display for RawFileError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RawFileError::Io(ref e) => {write!(f,"IO error: {}",e.description())},
            RawFileError::Utf8(ref e) => {write!(f,"Utf8 conversion error: {}",e.description())},
            RawFileError::FileFormat(ref s) => {write!(f,"File format error: {}",s)},
            RawFileError::Seek(p) => {write!(f,"Seek error: {}",p)},
            RawFileError::NotImplemented(ref s) => {write!(f,"Feature not Implemented: {}",s)}
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
    
    #[derive(Default)]
    pub struct RawImage {
        pub file_name:  Box<String>,
        byte_order: ByteOrder,
        pub ifd_offset: u32,
        tiff_offset: u32
    }


pub fn open(path: String) -> Result<RawImage,RawFileError>{

    let mut f = try!(File::open(&path));
    let mut ri: RawImage = Default::default();
    try!(ri.read_header(&mut f));
    ri.file_name = Box::new(String::from(path));
    Ok(ri)
}

impl RawImage {
fn read_header(&mut self,f: &mut File) -> Result<(),RawFileError> {
    
    if 0 != try!(f.seek(::std::io::SeekFrom::Start(0))) { return Err(RawFileError::Seek(0)) } ;
    let mut bo : [u8; 2] = [0,0];       // Byte Order
    try!(f.read(&mut bo));
    let s = try!(str::from_utf8(&bo));
    match s {
        "II" => self.byte_order = ByteOrder::Intel,
        "MM" => self.byte_order = ByteOrder::Motorola,
        _    => return Err(RawFileError::FileFormat("Unknown byte order".to_string())) 
    }
    if s != "II" { return Err(RawFileError::NotImplemented("Only Intel Byte Order!".to_string())) };
    
    let mut tm : [u8; 2] = [0; 2];         // Tiff Magic
    try!(f.read(&mut tm));    
    if unsafe{ mem::transmute::<[u8;2],u16>(tm)} != 0x002a { return Err(RawFileError::FileFormat("Tiff Magic mismatch".to_string()))};
        
    let mut to : [u8; 4] = [0; 4];        // Tiff Offset
    try!(f.read(&mut to));    
    unsafe { self.tiff_offset = mem::transmute::<[u8;4],u32>(to)};
    
    let mut cm : [u8; 2] = [0; 2];         // CR2 Magic
    try!(f.read(&mut cm));    
    if try!(str::from_utf8(&cm)) != "CR" { return Err(RawFileError::FileFormat("CR2 Magic mismatch".to_string()));}
    
    let mut cmaj : [u8; 1] = [0];        // CR2 Major
    try!(f.read(&mut cmaj));    
    let mut cmin : [u8; 1] = [0];        // CR2 Minor
    try!(f.read(&mut cmin));    
    if (cmaj ,cmin) != ([2],[0]) { return Err(RawFileError::FileFormat("Version not 2.0".to_string()));}
    
    let mut io : [u8; 4] = [0;4];         // IFD Offset
    try!(f.read(&mut io));    
    unsafe { self.ifd_offset = mem::transmute::<[u8;4],u32>(io)};
    Ok(())

}
}

