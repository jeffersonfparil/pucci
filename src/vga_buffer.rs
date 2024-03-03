// We store the colours as u8 although u4 is enough but Rust does not have u4
// We will fit all the colour propeties in u8.
// To do this we will perform a u8 shift such that:
//      - the foreground colour occupies the first 4 bits, and 
//      - the background colour will be shifted to to the last 4 bits via (brackground as u8) << 4 | (foreground as u8)
// #[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colour {
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

// We will store the full colour codes (foreground and background colours) in u8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColourCode(u8);

impl ColourCode {
    // Here we fit both foreground and background colours into 8 bits
    //      - We shift the background colour up to the last 4 bits,
    //      - and perform a bitwise or so that the foreground colour occupies the first 4 bits.
    fn new(foreground: Colour, background: Colour) -> ColourCode {
        ColourCode((background as u8) << 4 | (foreground as u8))
    }
}

// Characters to be printed on-screen will be u8 (ASCII, i.e. [code page 47](https://en.wikipedia.org/wiki/Code_page_437))
// We need to have the struct elements sorted as is and since Rust doesn't care of the order we use C's sorted struct layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    colour_code: ColourCode,
}

// Struct for the text buffer with the same type as its underlying element, i.e.
// two-dimensional array containing the C-style sorted element struct, ScreenChar.
// The first dimension refers to the row position and and second refers to the column position.
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
// #[repr(transparent)]
// struct Buffer {
//     chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
// }
// We revise the Buffer using the volatile crate
use volatile::Volatile;
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Struct for writing into the screen buffer
// We use 'static lifetime so that our reference to the screen buffer is valid for the entire program
pub struct Writer {
    column_position: usize,
    colour_code: ColourCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn new_line(&mut self) {
        /* TODO */
    }
    // We print each character (i.e. a byte) with the logic below for newlines and wrapping if we reach the edge of the screen buffer.
    // And also including the colours, and moving the column position by one each time we print a character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let colour_code = self.colour_code;
                // Modify the Buffer, i.e. write to it.
                // self.buffer.chars[row][col] = ScreenChar {
                //     ascii_character: byte,
                //     colour_code,
                // };
                // However, we need the "volatile" crate to prevent possible future compilation 
                //      which may get rid of the step below altogether because we do not read it, and
                //      it does not know we're writing into the VGA and not into RAM,
                //      and the compiler may optimise it away.
                // Below instead of directly mutating ScreenChar, we use the volatile::Volatile's write method
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    colour_code,
                });
                self.column_position += 1;
            }
        }
    }
    // We need to write strings one character (one byte at a time)
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // ASCII character (from space (32nd character) to tilde (126th character), i.e. 95 characters) or newline
                // See this [ASCII table](http://www.roysac.com/learn/ascii-table-ccu.htm)
                0x20..=0x73 | b'\n' => self.write_byte(byte),
                // For the other characters we simply print ■ (ASCII 0x00fe, the 254th character)
                _ => self.write_byte(0xfe),
            }
        }
    }
}

// Here we define Rust's formatting macros `write!` and `writeln!`, because they're nice and simple enough to implement,
//      i.e. simple define the write_str method within the core::fmt::Write trait
// This will allow us to use Rust's built-in write! and writeln!
use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}



// Test screen writing function
pub fn print_someshit() {
    use core::fmt::Write; // Use Rust's built-in formatting macros (`write!` and `writeln!`) which we implemented for our Writer struct above.
    let mut writer = Writer {
        column_position: 0,
        colour_code: ColourCode::new(Colour::Red, Colour::White),
        buffer: unsafe {&mut *(0xb8000 as *mut Buffer)},
    };
    writer.write_byte(b'H');
    writer.write_string("ello ");
    // writer.write_string("Wörld!");
    // Using Rust's built-in write! macro after implementing `write_str` method above for our Writer struct
    write!(writer, "World!\nNice numbers in my opinion are {} and {}.", 42.00000000000001, 789.0/123.0).expect("Error: something went wrong with writing our characters into VGA memory!");
    // Note that since we have not yet implemented the `new_line` method for our Writer struct then we're overflowing the first line.
}
