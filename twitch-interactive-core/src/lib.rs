use std::fs::File;
use std::mem;

use mmap_wrapper::{MmapMutWrapper, MmapWrapper};

#[repr(C)]
pub struct LatestStreamInfo {
    pub msgs_per_15s: u64,
    pub msgs_per_30s: u64,
    pub msgs_per_60s: u64,
    pub waters_per_10m: u64,
    pub raid: u64,   // set to true for first 3 minutes of raid or something?
    pub follow: u64, // see above^ but for 15 seconds?
    pub redeem: u64,
}

impl LatestStreamInfo {
    pub fn new_mut<P: AsRef<str>>(path: P) -> MmapMutWrapper<Self> {
        let f = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.as_ref())
            .unwrap();

        let _ = f.set_len(mem::size_of::<LatestStreamInfo>() as u64);

        let m = unsafe { memmap2::MmapMut::map_mut(&f).unwrap() };
        MmapMutWrapper::new(m)
    }

    pub fn new<P: AsRef<str>>(path: P) -> MmapWrapper<Self> {
        let f = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.as_ref())
            .unwrap();

        let _ = f.set_len(mem::size_of::<LatestStreamInfo>() as u64);

        let m = unsafe { memmap2::Mmap::map(&f).unwrap() };
        MmapWrapper::new(m)
    }
}
