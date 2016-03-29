extern crate raw;
use raw::cr2;
fn main() {
    let res = cr2::open("data/test.cr2".to_string());
    match res {
        Ok(ri) => { println!("File: {} Offset: {}",ri.file_name,ri.raw_offset);}
        Err(e) => { println!("Fehler {}",e);}
    }
}
