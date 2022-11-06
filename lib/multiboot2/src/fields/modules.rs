use super::{Tag, TagIter, TagType};

use core::ffi::{c_char, CStr};
use core::fmt::Debug;

/// This tag indicates to the kernel what boot module was loaded along with the kernel image, and where it can be found.
/// One tag appears per module. This tag type may appear multiple times.
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Module {
    tag: Tag,
    mod_start: u32,
    mod_end: u32,
    string: c_char, // slice of length size - 8, should be a null terminated UTF-8 string
}

impl Module {
    pub fn string(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr((&self.string) as *const i8) };
        // Must be valid UTF-8 occording to the spec
        c_str.to_str().unwrap()
    }

    /// The physical address to the start of the module
    pub fn mod_start(&self) -> usize {
        self.mod_start as usize
    }

    /// The physical address to the end of the module
    pub fn mod_end(self) -> usize {
        self.mod_end as usize
    }
}

impl Debug for Module {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct Hex(usize);
        impl Debug for Hex {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:#X}", self.0)
            }
        }

        f.debug_struct("Modules")
            .field("mod_start", &Hex(self.mod_start()))
            .field("mod_end", &Hex(self.mod_end()))
            .field("string", &self.string())
            .finish()
    }
}

/// An iterator over all module tags.
#[derive(Clone)]
pub struct ModuleIter<'a> {
    iter: TagIter<'a>,
}

impl<'a> ModuleIter<'a> {
    pub(crate) fn new(tag_iter: TagIter<'a>) -> Self {
        Self { iter: tag_iter }
    }
}

impl<'a> Iterator for ModuleIter<'a> {
    type Item = &'a Module;

    fn next(&mut self) -> Option<&'a Module> {
        self.iter
            .find(|x| x.tag_type() == TagType::Module)
            .map(|tag| unsafe { &*(tag as *const Tag as *const Module) })
    }
}

impl<'a> Debug for ModuleIter<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        self.clone().for_each(|tag| {
            list.entry(&tag);
        });
        list.finish()
    }
}
