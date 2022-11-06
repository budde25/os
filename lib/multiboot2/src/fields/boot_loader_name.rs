use super::Tag;

use core::ffi::{c_char, CStr};
use core::fmt::Debug;

#[repr(C, packed)]
pub struct BootLoaderName {
    tag: Tag,
    string: c_char, // slice of length size - 8, should be a null terminated UTF-8 string
}

impl BootLoaderName {
    pub fn string(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr((&self.string) as *const i8) };
        // Must be valid UTF-8 occording to the spec
        c_str.to_str().unwrap()
    }
}

impl Debug for BootLoaderName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderName")
            .field("tag", &self.tag)
            .field("string", &self.string())
            .finish()
    }
}
