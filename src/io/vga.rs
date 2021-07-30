use core::fmt;

const BUFFER_WIDTH: usize = 80; // default 80
const BUFFER_HEIGHT: usize = 25; // default 25

/// Hardware text mode color constants.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl Default for ColorCode {
    /// Classic terminal feel
    fn default() -> Self {
        Self::new(Color::Green, Color::Black)
    }
}

impl ColorCode {
    /// Create a color code out of a foreground and background color
    fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    ascii_character: u8,
    color_code: ColorCode,
}

impl Char {
    /// Create a new char
    fn new(ascii_character: u8, color_code: ColorCode) -> Self {
        Self {
            ascii_character,
            color_code,
        }
    }
}

#[repr(transparent)]
struct Buffer {
    chars: [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    unsafe fn get_mut_ptr(&mut self, row: usize, col: usize) -> *mut Char {
        self.chars[row].as_ptr().add(col) as *mut Char
    }
}

pub struct Vga {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Default for Vga {
    fn default() -> Self {
        Self {
            column_position: 0,
            color_code: ColorCode::default(),
            // Safety: 0xb8000 is the location of the VGA buffer
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }
}

impl Vga {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let screen_char = Char::new(byte, self.color_code);
                self.buffer.chars[row][col] = screen_char;
                unsafe {
                    self.buffer
                        .get_mut_ptr(row, col)
                        .write_volatile(screen_char)
                }
                self.column_position += 1;
            }
        }
    }

    /// Writes a string
    fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts everything up by one line
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    let character = self.buffer.get_mut_ptr(row, col).read_volatile();
                    self.buffer
                        .get_mut_ptr(row - 1, col)
                        .write_volatile(character);
                }
            }
        }
        // clear bottom row and reset col position
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// clear a row with empty char, uses the set background color
    fn clear_row(&mut self, row: usize) {
        let blank_char = Char::new(b' ', self.color_code);
        for col in 0..BUFFER_WIDTH {
            unsafe {
                self.buffer.get_mut_ptr(row, col).write_volatile(blank_char);
            }
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }
}

impl fmt::Write for Vga {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel_print;

    use super::super::VGA;

    #[test_case]
    fn test_println_simple() {
        VGA.lock().clear_screen();
        kernel_print!("test_vga_simple output");
    }

    #[test_case]
    fn test_println_many() {
        VGA.lock().clear_screen();
        for _ in 0..200 {
            kernel_print!("test_vga_many output");
        }
    }
}
