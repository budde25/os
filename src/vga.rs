extern "C" {
    static VGA_WIDTH: usize; // default 80
    static VGA_HEIGHT: usize; // default 25
}

// Hardware text mode color constants.
#[repr(C)]
enum VGAColor {
	Black = 0,
    Blue = 1,
	Green = 2,
	Cyan = 3,
	Red = 4,
	Magenta = 5,
	Brown = 6,
    LightGrey = 7,
	DarkGrey = 8,
	LightBlue = 9,
	LightGreen = 10,
	LightCyan = 11,
	LightRed = 12,
	LightMagenta = 13,
	LightBrown = 14,
	White = 15,
}