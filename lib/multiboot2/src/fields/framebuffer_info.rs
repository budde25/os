use super::Tag;

use core::fmt::Debug;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct FrameBufferInfo {
    tag: Tag,
    address: u64,
    pitch: u32,
    width: u32,
    height: u32,
    bpp: u8,
    framebuffer_type: FrameBufferType,
    _reserved: u8,
    color_info: [u8; 0], // color info data defined in structs below
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameBufferType {
    Indexed = 0,
    DirectRgb = 1,
}

impl FrameBufferInfo {
    /// The address to the framebuffer
    fn address(&self) -> usize {
        usize::try_from(self.address).unwrap()
    }

    fn pitch(&self) -> u32 {
        self.pitch
    }

    fn width(&self) -> u32 {
        self.pitch
    }

    fn height(&self) -> u32 {
        self.height
    }

    /// bits per pixel
    fn bpp(&self) -> u8 {
        self.bpp
    }

    fn framebuffer_type(&self) -> FrameBufferType {
        self.framebuffer_type
    }

    fn color_direct_rgb(&self) -> Option<&'static ColorDirectRgb> {
        if self.framebuffer_type != FrameBufferType::DirectRgb {
            return None;
        }
        let offset = 31;
        let ptr = self as *const FrameBufferInfo as *const u8;
        Some(unsafe { &*(ptr.add(offset) as *const ColorDirectRgb) })
    }

    fn color_indexed(&self) -> Option<&'static ColorIndexed> {
        if self.framebuffer_type != FrameBufferType::Indexed {
            return None;
        }
        let offset = 31;
        let ptr = self as *const FrameBufferInfo as *const u8;
        Some(unsafe { &*(ptr.add(offset) as *const ColorIndexed) })
    }
}

impl Debug for FrameBufferInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FrameBufferInfo")
            .field("address", &self.address())
            .field("pitch", &self.pitch())
            .field("width", &self.width())
            .field("height", &self.height())
            .field("bpp", &self.bpp())
            .field("type", &self.framebuffer_type())
            .finish_non_exhaustive()
    }
}

// types for framebuffer

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct ColorDirectRgb {
    red_field_position: u8,
    red_mask_size: u8,
    green_field_position: u8,
    green_mask_size: u8,
    blue_field_position: u8,
    blue_mask_size: u8,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct ColorIndexed {
    num_colors: u32,
}

impl ColorIndexed {
    fn pallets(&self) -> &'static [Pallet] {
        let offset = 4;
        let ptr = self as *const ColorIndexed as *const u8;
        let ptr = unsafe { ptr.add(offset) };
        let ptr = ptr as *const Pallet;
        unsafe { core::slice::from_raw_parts(ptr.add(offset), self.num_colors as usize) }
    }
}

impl Debug for ColorIndexed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let num_colors = self.num_colors;
        f.debug_struct("ColorIndexed")
            .field("num_colors", &num_colors)
            .field("pallets", &self.pallets())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct Pallet {
    red_value: u8,
    green_value: u8,
    blue_value: u8,
}
