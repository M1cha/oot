use std::io::{Seek, SeekFrom};
use std::path::Path;
use std::fs::File;
use std::sync::Arc;

mod generated {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
pub use generated::*;

pub struct DisplayList {

}

const FILE_TABLE_OFFSET: u32 = 0x12F70;

fn load_offset(baserom: &mut File, i: u32) -> anyhow::Result<u32> {
    let offset = FILE_TABLE_OFFSET + i * 16;
    baserom.seek(SeekFrom::Start(offset.into()))?;

    Ok(0)
}

fn load_texture(baserom: &mut File, offset: u32) -> anyhow::Result<Arc<Vec<u8>>> {
    Ok(Arc::new(Vec::new()))
}

fn load_displaylist(baserom: &mut File, offset: u32) -> anyhow::Result<Arc<DisplayList>> {
    Ok(Arc::new(DisplayList{}))
}

fn load_stub(baserom: &mut File, offset: u32) -> anyhow::Result<()> {
    Ok(())
}

pub trait Load {
    fn load(baserom: &mut std::fs::File) -> anyhow::Result<Self> where Self:Sized; 
}

pub struct AssetManager {
    baserom: File,
}

impl AssetManager {
    pub fn new<P: AsRef<Path>>(baserom_path: P) -> std::io::Result<Self> {
        Ok(Self {
            baserom: std::fs::File::open(baserom_path.as_ref())?,
        })
    }

/*    pub fn load_file<T:Load>(&mut self, file: File) -> anyhow::Result<T> {
        T::load(&mut self.baserom)
    }*/
}
