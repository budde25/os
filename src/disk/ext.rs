#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Superblock {
    inodes_count: u32,
    blocks_count: u32,
    r_blocks_clount: u32, // number of blocks reserved for superuser
    free_block_count: u32,
    free_inode_count: u32,
    first_data_block: u32,
    log_block_size: u32,   // left shift 1024 by number
    log_frag_size: i32,    // left shift 1024 by number
    blocks_per_group: u32, // number of blocks in each block group
    frags_per_groups: u32, // number of inoes in each block group
    inodes_per_group: u32, // number of fragments for each block group
    mtime: u32,            // POSIX last mount time
    wtime: u32,            // POSIX last written tim
    mnt_count: u16, // number of times the volume has been mounted since its last consistency check (fsck)
    max_mnt_count: u16, // number of mounts allowed before a consistency check (fsck) must be done
    magic: u16,     // 0xef53
    state: FsStates, // file system state
    errors: Errors, // what to do if an error is detected
    minor_rev_level: u16,
    lastcheck: u32,        // POSIX
    checkinterval: u32,    // POSIX interval between forced check
    creator_os: CreatorOs, // os id that created volume
    rev_level: u32,
    def_resuid: u16, // user id that can use reserved blocks
    def_resgid: u16, // group id that can use reserved blocks
    // extended only if major version >= 1
    first_inode: u32,                   // first non reserved inode
    inode_size: u16,                    // size of inode struct in bytes
    block_group_nr: u16,                // of this superblock (if backup)
    feature_compat: FeatureCompat,      // not required to support read or write
    feature_incompat: FeatureIncompat,  // required to support read or write
    feature_ro_compat: FeatureRoCompat, // if not supported must be read only
    uuid: u128,
    volume_name: u128,      // cstring
    last_mounted: [u8; 64], // cstring, path volume was last mounted to
    algo_bitmap: u32,       // compression used
    // performance hints
    prealloc_blocks: u8,     // number of blocks to preallocate for files
    prealloc_dir_blocks: u8, // number of blocks to preallocate for dirs
    _reserved_1: u16,        // alignment
    // jounaling support
    journal_uuid: u128, // same style as file system id
    journal_inum: u32,
    journal_dev: u32,
    last_orphan: u32, // head of orphan inode list
    // directory indexing support
    hash_seed: [u32; 4],
    def_hash_version: u8,
    _reserved_2: [u8; 3], // padding reserved for expansion
    // other
    default_mnt_opts: u32, // deafult mount options
    first_meta_bg: u32,

    _reserved_3: [u8; 760], // unused
}

impl Superblock {
    // check magic ext2 value
    fn verify(&self) -> bool {
        self.magic == 0xEF53
    }

    fn volume_name(&self) -> &str {
        let volume_name = self.volume_name;
        let ptr = &volume_name as *const _ as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, core::mem::size_of::<u128>()) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    fn block_size(&self) -> u64 {
        1024 << self.log_block_size as u64
    }

    fn frag_size(&self) -> u64 {
        if self.log_frag_size >= 0 {
            1024 << self.log_frag_size as u64
        } else {
            1024 >> (-self.log_frag_size) as u64
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
enum FsStates {
    Valid = 1,  // unmounted cleanly
    Errors = 2, // errors detected
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
enum Errors {
    Continue = 1, // ignore errors
    Ro = 2,       // remount ro
    Panic = 3,    // kernel panic
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum CreatorOs {
    Linux = 0,
    Hurd = 1,
    Masix = 2,
    FreeBsd = 3,
    Lites = 4,
}

bitflags::bitflags! {
    struct FeatureCompat: u32 {
        const DIR_PREALLOC = 0x1;
        const IMAGIC_INODES = 0x2;
        const HAS_JOURNAL = 0x4;
        const EXT_ATTR = 0x8;
        const RESIZE_INO = 0x10;
        const DIR_INDEX = 0x20;
    }
}

bitflags::bitflags! {
    struct FeatureIncompat: u32 {
        const COMPRESSION = 0x1;
        const FILETYPE = 0x2;
        const RECOVER = 0x4;
        const JOURNAL_DEV = 0x8;
        const META_BG = 0x10;
    }
}

bitflags::bitflags! {
    struct FeatureRoCompat: u32 {
        const SPARSE_SUPER = 0x1; // sparse superblock
        const LARGE_FILE = 0x2; // large file support, 64bit file size
        const BTREE_DIR = 0x4; // binay tree sorted dir files
    }
}

bitflags::bitflags! {
    struct AlgoBitmap: u32 {
        const LZV1 = 0x1;
        const LZRW3A = 0x2;
        const GZIP = 0x4;
        const BZIP2 = 0x8;
        const LZO = 0x10;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]

struct BlockGroupDescriptorTable {
    block_bitmap: u32,
    inode_bitmap: u32,
    inode_table: u32,
    free_block_count: u16,
    free_inode_count: u16,
    used_dirs_count: u16,
    _padding: u16,
    _reserved: [u8; 12],
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct InodeTable {
    mode: Imode,
    uid: u16,
    size: u32,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    gid: u16,
    links_count: u16,
    blocks: u32,
    flags: Iflags,
    osd1: u32,
    block: [u32; 15],
    generation: u32,
    file_acl: u32,
    dir_acl: u32,
    faddr: u32,
    osd2: [u8; 12],
}

enum ReservedInodes {
    Bad = 1,
    Root = 2,
    AclInd = 3,
    AclData = 4,
    BootLoader = 5,
    UndelDir = 6,
}

bitflags::bitflags! {
    struct Imode: u16 {
        // access rights
        const XOTH  = 0x0001; // others execute
        const WOTH  = 0x0002; // others write
        const ROTH  = 0x0004; // others read
        const XGRP  = 0x0008; // group execute
        const WGRP  = 0x0010; // group write
        const RGRP  = 0x0020; // group read
        const XUSR  = 0x0040; // user execute
        const WUSR  = 0x0080; // user write
        const RUSR  = 0x0100; // user read
        // process exection user/group override
        const SVTX  = 0x0200; // stick bit
        const SGID  = 0x0400; // set process group id
        const SUID  = 0x0800; // set process user id
        // file format
        const FIFI  = 0x1000; // fifo
        const FCHR  = 0x2000; // character device
        const FDIR  = 0x4000; // directory
        const FBLK  = 0x6000; // block device
        const FREG  = 0x8000; // regular file
        const FLNK  = 0xA000; // symbolic link
        const FSOCK = 0xC000; // socket
    }
}

bitflags::bitflags! {
    struct Iflags: u32 {
        const SECRM        = 0x00000001; // secure deletion
        const UNRM         = 0x00000002; // recored to undelete
        const COMPR        = 0x00000004; // compress file
        const SYNC         = 0x00000008; // synchronous updates
        const IMMUTABLE    = 0x00000010; // immutable file
        const APPEND       = 0x00000020; // append only
        const NODUMP       = 0x00000040; // do not dump / delete file
        const NOATIME      = 0x00000080; // do not update i_atime
        // reserved for compression
        const DIRTY        = 0x00000100; // dity (modified)
        const COMPRBLK     = 0x00000200; // compressd blocks
        const NOCOMPR      = 0x00000400; // access raw compressed data
        const ECOMPR       = 0x00000800; // compression error
        // end compressrion flags
        const BTREE        = 0x00001000; // b-tree formate directory
        const INDEX        = 0x00001000; // hash indexed directory
        const IMAGIC       = 0x00002000; // AFS directory
        const JOURNAL_DATA = 0x00004000; // journal file data
        const RESERVED     = 0x80000000; // reserved for ext2 library
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct LinkedDirectoryEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: FileType,
    name: [u8; 0], // variable len 0-255 based on name_len
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum FileType {
    Unknown = 0, // unknown file type
    RegFile = 1, // regular file
    Dir = 2,     // directory file
    Chrdev = 3,  // character device
    Blkdev = 4,  // block device
    Fifo = 5,    // buffer file
    Sock = 6,    // socket file
    Symlink = 7, // symbolic link
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn superblock_size() {
        use core::mem::size_of;
        assert_eq!(size_of::<Superblock>(), 1024)
    }
}
