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
        pub raw_offset: u32,
        ifd_offsets: Vec<u32>
    }


pub fn open(path: String) -> Result<RawImage,RawFileError>{

    let mut f = try!(File::open(&path));
    let mut ri: RawImage = Default::default();
    try!(ri.read_header(&mut f));
    let mut i=0;
    while ri.ifd_offsets.len() > i {
        try!(ri.read_ifd(&mut f,i,false));
        i += 1;
    }
    ri.file_name = Box::new(String::from(path));
    Ok(ri)
}

impl RawImage {
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
    
    let mut tm = [0u8; 2];
    tm.clone_from_slice(&head[2..4]);         // Tiff Magic
    if unsafe{ mem::transmute::<[u8;2],u16>(tm)} != 0x002a { return Err(RawFileError::FileFormat("Tiff Magic mismatch".to_string()))};
        
    let mut to = [ 0u8; 4];        // Tiff Offset
    to.clone_from_slice(&head[4..8]);
    unsafe { self.ifd_offsets.push(mem::transmute::<[u8;4],u32>(to))};
    
    let cm = &head[8..10];         // CR2 Magic
    if try!(str::from_utf8(&cm)) != "CR" { return Err(RawFileError::FileFormat("CR2 Magic mismatch".to_string()));}
    
    let cmaj = &head[10..11];        // CR2 Major
    let cmin = &head[11..12];        // CR2 Minor
    if cmaj[0]!=2 && cmin[0]!=0 { return Err(RawFileError::NotImplemented(format!("CR2 Version {}.{} not supported",cmaj[0],cmin[0])));}
    
    let mut io = [0u8;4];         // IFD Offset
    io.clone_from_slice(&head[12..16]);
    unsafe { self.raw_offset = mem::transmute::<[u8;4],u32>(io)};
    Ok(())

}
fn read_ifd(&mut self,f: &mut File, index: usize,read_tags:bool) -> Result<u32,RawFileError>{
    let mut pos = try!(f.seek(io::SeekFrom::Start(self.ifd_offsets[index] as u64)));
    let mut na=[0u8; 2];
    try!(f.read(&mut na));
    let n = unsafe{ mem::transmute::<[u8;2],u16>(na)};
    if read_tags {
    }
    pos=pos+n as u64 *12+2;
    let mut ioa = [0u8; 4];
    try!(f.seek(io::SeekFrom::Start(pos)));
    try!(f.read(&mut ioa));
    let io = unsafe{ mem::transmute::<[u8;4],u32>(ioa)};    
    if io != 0 {self.ifd_offsets.push(io)};

    Ok(io)

}
}

