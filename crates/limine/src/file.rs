use core::{
    ffi::{CStr, c_char},
    ptr::NonNull,
};

#[repr(C)]
pub struct File {
    pub revision: u64,
    /// The address of the file.
    /// Always 4KiB aligned.
    address: u64,
    /// The size of the file.
    size: u64,
    /// The path of the file, with a leading `/`.
    path: *const c_char,
    /// The command line passed to the file.
    cmdline: *const c_char,
    /// The media type of the file.
    media_type: u32,
    unused: u32,
    /// If non-zero, the TFTP server IP address.
    tftp_ip: u32,
    /// Likewise, the TFTP server port.
    tftp_port: u32,
    /// 1-based partition index of the volume.
    /// 0 means not partitioned.
    partition_index: u32,
    /// If non-zero, the MBR disk ID.
    mbr_disk_id: u32,
    /// If non-zero, the GPT disk UUID.
    gpt_disk_uuid: Uuid,
    /// If non-zero, the GPT partition UUID.
    gpt_partition_uuid: Uuid,
    /// If non-zero, the filesystem UUID.
    part_uuid: Uuid,
}

impl File {
    pub fn path(&self) -> &str {
        // SAFETY: The path pointer is valid because it is a pointer to a string literal.
        unsafe { CStr::from_ptr(self.path).to_str().unwrap() }
    }

    pub fn cmdline(&self) -> &str {
        // SAFETY: The cmdline pointer is valid because it is a pointer to a string literal.
        unsafe { CStr::from_ptr(self.cmdline).to_str().unwrap() }
    }

    pub fn media_type(&self) -> MediaType {
        // SAFETY: The media type is a valid enum variant (guaranteed by the protocol).
        unsafe { core::mem::transmute(self.media_type) }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    Generic = 0,
    Optical = 1,
    Tftp = 2,
}

#[repr(C)]
struct Uuid {
    a: u32,
    b: u16,
    c: u16,
    d: [u8; 8],
}

pub struct FileIter<'a> {
    revision: u64,
    files: &'a [NonNull<File>],
    index: usize,
}

impl<'a> FileIter<'a> {
    pub(crate) fn new(revision: u64, files: &'a [NonNull<File>]) -> Self {
        Self {
            revision,
            files,
            index: 0,
        }
    }
}

impl<'a> Iterator for FileIter<'a> {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.files.len() {
            return None;
        }
        self.index += 1;
        // SAFETY: The file pointer is valid because it is a pointer to a file.
        Some(unsafe { self.files[self.index - 1].read_volatile() })
    }
}
