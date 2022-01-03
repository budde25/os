use core::fmt::Debug;

#[repr(C)]
pub struct Bitmap<const N: usize> {
    bitmap: [u8; N],
}

impl<const N: usize> Bitmap<N> {
    pub const fn new() -> Self {
        Self { bitmap: [0; N] }
    }

    pub const fn is_set(&self, index: usize) -> bool {
        assert!(index < N * 8);
        let byte_index = index / 8;
        let bit_index = index % 8;

        let byte = self.bitmap[byte_index];
        byte & (1 << bit_index) != 0
    }

    pub fn bit_set(&mut self, index: usize) {
        assert!(index < N * 8);
        let byte_index = index / 8;
        let bit_index = index % 8;

        let byte = self.bitmap[byte_index];
        self.bitmap[byte_index] = byte | (1 << bit_index)
    }

    pub fn bit_clear(&mut self, index: usize) {
        assert!(index < N * 8);
        let byte_index = index / 8;
        let bit_index = index % 8;

        let byte = self.bitmap[byte_index];
        self.bitmap[byte_index] = byte & !(1 << bit_index)
    }
}

impl<const N: usize> Default for Bitmap<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Debug for Bitmap<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct Byte(u8);
        impl Debug for Byte {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:08b}", self.0)
            }
        }

        let mut ds = f.debug_struct("Bitmap");
        for i in self.bitmap {
            let byte = Byte(i);
            ds.field("byte", &byte);
        }

        ds.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut empty = Bitmap::<10>::new();

        for i in 0..10 * 8 {
            assert_eq!(empty.is_set(i), false);
        }

        for i in 0..10 * 8 {
            empty.bit_set(i)
        }

        for i in 0..10 * 8 {
            assert_eq!(empty.is_set(i), true);
        }
    }

    #[test]
    fn test_single() {
        let mut empty = Bitmap::<10>::new();

        assert_eq!(empty.is_set(10), false);
        empty.bit_set(10);

        for i in 0..10 * 8 {
            if i == 10 {
                assert_eq!(empty.is_set(i), true);
            } else {
                assert_eq!(empty.is_set(i), false);
            }
        }
    }

    #[test]
    fn test_size() {
        use core::mem::size_of;
        assert_eq!(size_of::<Bitmap<1>>(), 1);
        assert_eq!(size_of::<Bitmap<10>>(), 10);
    }
}
