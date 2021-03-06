use core::mem::transmute_copy;

use bitmap::Bitmap;
use custom_debug_derive::Debug;
use staticvec::StaticVec;

use crate::consts::BSIZE;
use crate::disk::bcache;

const BLOCK_SIZE: usize = 1024;
const INODE_SIZE: usize = 256;

pub fn ext_test() {
    use crate::kdbg;

    let ext = Ext::new(1);
    //kdbg!(ext.block_groups[0].block_bitmap(1));
    //kdbg!(ext.block_groups[0].inode_bitmap(1));
    let i = ext.read_inode(2);
    kdbg!(i);
}

pub struct Ext {
    device: u32,
    superblock: Superblock,
    block_groups: StaticVec<BlockGroupDescriptor, 10>,
}

impl Ext {
    pub fn new(device: u32) -> Self {
        let superblock = Superblock::read(device);
        let mut block_groups = StaticVec::new();

        // we are starting at 1-2 empty, 2-3 superblock
        // TODO: this is stupid and slow
        for i in 0..superblock.block_groups() {
            block_groups.push(BlockGroupDescriptor::new(device, i as u8))
        }

        // assert we have a valid ext2 superblock
        assert_eq!(superblock.verify(), true);
        // FIXME: core logic relies on this being the same.
        assert_eq!(
            core::mem::size_of::<Inode>(),
            superblock.inode_size as usize
        );
        assert_eq!(superblock.block_size() as usize, BLOCK_SIZE);
        // must be on version 1 or newer of ext2
        assert!(superblock.rev_level >= 1);

        //crate::kdbg!(superblock);
        //crate::kdbg!(&block_groups);

        Self {
            device,
            superblock,
            block_groups,
        }
    }

    fn read_root_dir(&self) {
        const ROOT_INODE: usize = 2;
        self.read_inode(2);
    }

    fn read_inode(&self, inode: u32) -> Minode {
        use super::BUFFERS;
        let block_no = self.block_containing_inode(inode);

        let block_group = self.block_group_containing_inode(inode);
        let inode_table = block_group.inode_table;

        // we know the starting block, 4 inodes fit into one block
        let disk_block = inode_table + block_no;
        let disk_index = (inode - 1) % 4;

        let mut bufs = BUFFERS.lock();
        let buf = bufs.read(self.device, disk_block);
        let b = buf.borrow();

        let mut inode_raw: [u8; 256] = [0; 256];
        for i in 0..inode_raw.len() {
            inode_raw[i] = b.data()[i + (256 * disk_index as usize)];
        }

        let node = unsafe { core::intrinsics::transmute::<[u8; 256], Inode>(inode_raw) };
        node.into_minode(self.device, inode)
    }

    /// zero a block
    fn block_zero(&self, block_no: u32) {
        use super::BUFFERS;
        let mut bufs = BUFFERS.lock();
        let b = bufs.read(self.device, block_no);
        *b.borrow_mut().data_mut() = [0; BSIZE];
        bcache::BufferCache::write(b);
    }

    fn block_group_containing_inode(&self, inode: u32) -> &BlockGroupDescriptor {
        let block_group = (inode - 1) / self.superblock.inodes_per_group;
        &self.block_groups[block_group as usize]
    }

    fn block_containing_inode(&self, inode: u32) -> u32 {
        let index = (inode - 1) % self.superblock.inodes_per_group;
        let containing_block =
            (index * self.superblock.inode_size as u32) / self.superblock.block_size() as u32;
        containing_block
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Superblock {
    pub inodes_count: u32,
    pub blocks_count: u32,
    pub r_blocks_count: u32, // number of blocks reserved for superuser
    pub free_block_count: u32,
    pub free_inode_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,   // left shift 1024 by number
    pub log_frag_size: i32,    // left shift 1024 by number
    pub blocks_per_group: u32, // number of blocks in each block group
    pub frags_per_groups: u32, // number of inoes in each block group
    pub inodes_per_group: u32, // number of fragments for each block group
    pub mtime: u32,            // POSIX last mount time
    pub wtime: u32,            // POSIX last written tim
    pub mnt_count: u16, // number of times the volume has been mounted since its last consistency check (fsck)
    pub max_mnt_count: u16, // number of mounts allowed before a consistency check (fsck) must be done
    pub magic: u16,         // 0xef53
    pub state: FsStates,    // file system state
    pub errors: Errors,     // what to do if an error is detected
    pub minor_rev_level: u16,
    pub lastcheck: u32,        // POSIX
    pub checkinterval: u32,    // POSIX interval between forced check
    pub creator_os: CreatorOs, // os id that created volume
    pub rev_level: u32,
    pub def_resuid: u16, // user id that can use reserved blocks
    pub def_resgid: u16, // group id that can use reserved blocks
    // extended only if major version >= 1
    pub first_inode: u32,                   // first non reserved inode
    pub inode_size: u16,                    // size of inode struct in bytes
    pub block_group_nr: u16,                // of this superblock (if backup)
    pub feature_compat: FeatureCompat,      // not required to support read or write
    pub feature_incompat: FeatureIncompat,  // required to support read or write
    pub feature_ro_compat: FeatureRoCompat, // if not supported must be read only
    pub uuid: u128,
    pub volume_name: u128,      // cstring
    pub last_mounted: [u8; 64], // cstring, path volume was last mounted to
    pub algo_bitmap: u32,       // compression used
    // performance hints
    pub prealloc_blocks: u8,     // number of blocks to preallocate for files
    pub prealloc_dir_blocks: u8, // number of blocks to preallocate for dirs
    #[debug(skip)]
    _reserved_1: u16, // alignment
    // jounaling support
    pub journal_uuid: u128, // same style as file system id
    pub journal_inum: u32,
    pub journal_dev: u32,
    pub last_orphan: u32, // head of orphan inode list
    pub hash_seed: [u32; 4],
    pub def_hash_version: u8,
    #[debug(skip)]
    _reserved_2: [u8; 3], // padding reserved for expansion
    pub default_mnt_opts: u32, // deafult mount options
    pub first_meta_bg: u32,
    #[debug(skip)]
    _reserved_3: [u8; 760], // unused
}

impl Superblock {
    /// Read the superblock from the disk
    pub fn read(device: u32) -> Self {
        use core::mem::transmute;
        let mut b = super::BUFFERS.lock();
        // the superblock will in block 1
        let data = b.read(device, 1).borrow().data().clone();
        unsafe { transmute::<[u8; 1024], Superblock>(data) }
    }

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

    fn block_groups(&self) -> u64 {
        (self.blocks_count.div_ceil(self.blocks_per_group)).into()
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

#[derive(Debug)]
#[repr(C)]

struct BlockGroupDescriptor {
    block_bitmap: u32,
    inode_bitmap: u32,
    inode_table: u32,
    free_block_count: u16,
    free_inode_count: u16,
    used_dirs_count: u16,
    #[debug(skip)]
    _padding: u16,
    #[debug(skip)]
    _reserved: [u8; 12],
}

impl BlockGroupDescriptor {
    pub fn new(device: u32, index: u8) -> Self {
        use core::mem::transmute;

        let size = core::mem::size_of::<Self>();
        assert_eq!(size, 32);
        assert!((size * index as usize) < 512);

        let mut b = super::BUFFERS.lock();
        // the group descriptor will in block 4
        let block = b.read(device, 2).borrow();

        let mut bgd_raw: [u8; 32] = [0; 32];
        let data = block.data();
        for i in 0..size {
            bgd_raw[i] = data[i + (size * index as usize)];
        }

        unsafe { transmute::<[u8; 32], Self>(bgd_raw) }
    }

    pub fn block_bitmap(&self, device: u32) -> Bitmap<BSIZE> {
        let mut b = super::BUFFERS.lock();
        let block = b.read(device, self.block_bitmap);
        unsafe { transmute_copy(block.borrow().data()) }
    }

    pub fn inode_bitmap(&self, device: u32) -> Bitmap<BSIZE> {
        let mut b = super::BUFFERS.lock();
        let block = b.read(device, self.inode_bitmap);
        unsafe { transmute_copy(block.borrow().data()) }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Inode {
    mode: Imode,
    uid: u16,
    size: u32,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    gid: u16,
    links_count: u16,
    blocks: u32, // amount of 512 byte blocks of data this inode uses
    flags: Iflags,
    #[debug(skip)]
    _reserved_1: u32, // os specific, reserved on linux reserved for use
    direct_blocks: [u32; 12],
    indirect_block: u32,
    double_indirect_block: u32,
    triple_indirect_block: u32,
    generation: u32,
    file_acl: u32,
    dir_acl: u32, // file size high, for files zero otherwise (on rev >= 1)
    faddr: u32,
    // os specific values, we will copy linux
    #[debug(skip)]
    _reserved_2: u32, // u8: frag_num, u8: fragsize
    uid_hi: u32,
    gid_hi: u32,
    #[debug(skip)]
    _reserved_3: u64,
    #[debug(skip)]
    _reserved_4: [u8; 120], // padding FIXME: should not be here but makes our simple case easier
}

impl Inode {
    /// add data to convert this to an in memory inode (Minode)
    fn into_minode(self, device: u32, inum: u32) -> Minode {
        Minode {
            device,
            inum,
            ref_cnt: 1, // TODO: verify we should be at 1?
            valid: true,
            dirty: false,
            // transfer the rest
            mode: self.mode,
            uid: self.uid as u64 | ((self.uid_hi as u64) << 16),
            gid: self.gid as u64 | ((self.gid_hi as u64) << 16),
            size: self.size as u64 | ((self.dir_acl as u64) << 32),
            atime: self.atime,
            ctime: self.ctime,
            mtime: self.mtime,
            dtime: self.dtime,
            links_count: self.links_count,
            flags: self.flags,
            blocks: self.blocks,
            direct_blocks: self.direct_blocks,
            indirect_block: self.indirect_block,
            double_indirect_block: self.double_indirect_block,
            triple_indirect_block: self.triple_indirect_block,
            file_acl: self.file_acl,
        }
    }
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
    pub struct Imode: u16 {
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
    pub struct Iflags: u32 {
        const SECRM        = 0x00000001; // secure deletion
        const UNRM         = 0x00000002; // record to undelete
        const COMPR        = 0x00000004; // compress file
        const SYNC         = 0x00000008; // synchronous updates
        const IMMUTABLE    = 0x00000010; // immutable file
        const APPEND       = 0x00000020; // append only
        const NODUMP       = 0x00000040; // do not dump / delete file
        const NOATIME      = 0x00000080; // do not update i_atime
        // reserved for compression
        const DIRTY        = 0x00000100; // dirty (modified)
        const COMPRBLK     = 0x00000200; // compressed blocks
        const NOCOMPR      = 0x00000400; // access raw compressed data
        const ECOMPR       = 0x00000800; // compression error
        // end compression flags
        const BTREE        = 0x00001000; // b-tree format directory
        const INDEX        = 0x00001000; // hash indexed directory
        const IMAGIC       = 0x00002000; // AFS directory
        const JOURNAL_DATA = 0x00004000; // journal file data
        const RESERVED     = 0x80000000; // reserved for ext2 library
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct LinkedDirectoryEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: FileType,
    name: [u8; 0], // variable len 0-255 based on name_len
}

impl LinkedDirectoryEntry {
    fn name(&self) -> &'static str {
        let ptr = &self.name as *const u8;
        let name_slice = unsafe { core::slice::from_raw_parts(ptr, self.name_len as usize) };
        unsafe { core::str::from_utf8_unchecked(name_slice) }
    }

    fn next_entry(&self) -> Option<Self> {
        let ptr = self as *const Self as *mut u8;
        let ptr = unsafe { ptr.add(self.rec_len as usize) };
        // maybe?

        unsafe { Some(*(ptr as *const Self)) }
    }
    // TODO: implement intoiterator
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

// In memory copy of an inode
#[derive(Debug, Clone)]
struct Minode {
    device: u32,
    inum: u32, // inode number
    ref_cnt: u32,
    valid: bool,
    dirty: bool,
    // inode relevant data
    mode: Imode,
    uid: u64,
    gid: u64,
    size: u64,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    links_count: u16,
    flags: Iflags,
    blocks: u32, // amount of 512 byte blocks of data this inode uses
    direct_blocks: [u32; 12],
    indirect_block: u32,
    double_indirect_block: u32,
    triple_indirect_block: u32,
    file_acl: u32,
}

impl Minode {
    pub fn invaid() -> Self {
        Self {
            valid: false,
            device: 0,
            inum: 0,
            ref_cnt: 0,
            dirty: false,
            // inode specific
            mode: Imode::empty(),
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            ctime: 0,
            mtime: 0,
            dtime: 0,
            links_count: 0,
            flags: Iflags::empty(),
            blocks: 0,
            direct_blocks: [0; 12],
            indirect_block: 0,
            double_indirect_block: 0,
            triple_indirect_block: 0,
            file_acl: 0,
        }
    }
}

impl Into<Inode> for Minode {
    fn into(self) -> Inode {
        Inode {
            mode: self.mode,
            uid: (self.uid & 0xFF) as u16,
            size: (self.size & 0xFFFF) as u32,
            atime: self.atime,
            ctime: self.ctime,
            mtime: self.mtime,
            dtime: self.dtime,
            gid: (self.gid & 0xFF) as u16,
            links_count: self.links_count,
            blocks: self.blocks,
            flags: self.flags,
            _reserved_1: 0,
            direct_blocks: self.direct_blocks,
            indirect_block: self.indirect_block,
            double_indirect_block: self.double_indirect_block,
            triple_indirect_block: self.triple_indirect_block,
            generation: 0,
            file_acl: self.file_acl,
            dir_acl: (self.size >> 32) as u32,
            faddr: 0,
            _reserved_2: 0,
            uid_hi: (self.uid >> 16) as u32,
            gid_hi: (self.gid >> 16) as u32,
            _reserved_3: 0,
            _reserved_4: [0; 120],
        }
    }
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
