//! Types for representing files.
use core::{
    ffi::{CStr, c_char},
    num::NonZeroU32,
    ptr::NonNull,
};

/// A file that is passed to the kernel.
///
/// This represents a file on the filesystem.
#[repr(C)]
pub struct File {
    /// The revision of the file.
    pub revision: u64,
    /// The address of the file.
    /// Always 4KiB aligned.
    pub address: u64,
    /// The size of the file.
    pub size: u64,
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
    /// Returns the path of the file with the leading `/`.
    pub fn path(&self) -> &str {
        // SAFETY: The path pointer is valid because it is a pointer to a string literal.
        unsafe { CStr::from_ptr(self.path).to_str().unwrap() }
    }

    /// Returns the command line passed to the file.
    pub fn cmdline(&self) -> &str {
        // SAFETY: The cmdline pointer is valid because it is a pointer to a string literal.
        unsafe { CStr::from_ptr(self.cmdline).to_str().unwrap() }
    }

    /// Returns the media type of the file.
    /// See [`MediaType`] for more information.
    pub fn media_type(&self) -> MediaType {
        assert!(self.media_type <= 2, "corrupt media type!");
        // SAFETY: The media type is a valid enum variant (guaranteed by the protocol).
        unsafe { core::mem::transmute(self.media_type) }
    }

    /// Returns the TFTP server IP address and port.
    pub fn tftp_info(&self) -> Option<(u32, u32)> {
        if self.tftp_ip != 0 {
            Some((self.tftp_ip, self.tftp_port))
        } else {
            None
        }
    }

    /// Returns the partition index of the partition the file is on
    pub fn partition_index(&self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.partition_index)
    }

    // TODO: MBR, GPT, Part info
}

impl core::fmt::Debug for File {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File")
            .field("revision", &self.revision)
            .field("address", &self.address)
            .field("size", &self.size)
            .field("path", &self.path())
            .field("cmdline", &self.cmdline())
            .field("media_type", &self.media_type())
            .field("tftp_info", &self.tftp_info())
            .field("partition_index", &self.partition_index())
            .field("mbr_disk_id", &self.mbr_disk_id)
            .field("gpt_disk_uuid", &self.gpt_disk_uuid)
            .field("gpt_partition_uuid", &self.gpt_partition_uuid)
            .field("part_uuid", &self.part_uuid)
            .finish()
    }
}

/// The media type of a file.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    /// A generic file.
    Generic = 0,
    /// A floppy disk.
    Optical = 1,
    /// A TFTP server.
    Tftp = 2,
}

/// A UUID.
#[repr(C)]
struct Uuid {
    a: u32,
    b: u16,
    c: u16,
    d: [u8; 8],
}

impl core::fmt::Debug for Uuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Uuid")
            .field(&format_args!(
                "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                self.a,
                self.b,
                self.c,
                self.d[0],
                self.d[1],
                self.d[2],
                self.d[3],
                self.d[4],
                self.d[5],
                self.d[6],
                self.d[7]
            ))
            .finish()
    }
}

/// An iterator over the files.
pub struct FileIter<'a> {
    files: &'a [NonNull<File>],
    index: usize,
}

impl<'a> FileIter<'a> {
    /// Creates a new `FileIter` from a slice of file pointers.
    pub(crate) fn new(files: &'a [NonNull<File>]) -> Self {
        Self { files, index: 0 }
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
