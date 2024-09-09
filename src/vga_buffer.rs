use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// 使用 #[allow(dead_code)]，可以对 Color 枚举类型禁用"未使用变量"警告。
#[allow(dead_code)]
// 生成trait：让类型遵循复制语义（copy semantics），也让它可以被比较、被调试和打印。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 用 repr(u8) 注记标注的枚举类型，都会以一个 u8 的形式存储——事实上 4 个二进制位就足够了，但 Rust 语言并不提供 u4 类型。
#[repr(u8)]
pub enum Color {
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

// 生成 Copy 和 Debug 等一系列 trait
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 使ColorCode 和 u8 有完全相同的内存布局
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 使用 #[repr(C)] 标记结构体；这将按 C 语言约定的顺序布局它的成员变量，让我们能正确地映射内存片段。
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// 对 Buffer 类型，我们再次使用 repr(transparent)，来确保类型和它的单个成员有相同的内存布局。
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
	// column_position 变量将跟踪光标在最后一行的位置。
    column_position: usize,  
	// 当前字符的前景和背景色将由 color_code 变量指定；
    color_code: ColorCode,
	// 使用 'static 生命周期（’static lifetime），意味着这个借用应该在整个程序的运行期间有效；
	// 这对一个全局有效的 VGA 字符缓冲区来说，是非常合理的。
    buffer: &'static mut Buffer,
}

// 将让这个 Writer 类型将字符写入屏幕的最后一行，并在一行写满或接收到换行符 \n 的时候，将所有的字符向上位移一行。
/*
如果这个字节是一个换行符（line feed）字节 \n，我们的 Writer 不应该打印新字符，
相反，它将调用我们稍后会实现的 new_line 方法；
其它的字节应该将在 match 语句的第二个分支中被打印到屏幕上。
*/
impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
				// 检查当前行是否已满。如果已满，它将首先调用 new_line 方法来将这一行字向上提升
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
				// 再将一个新的 ScreenChar 写入到缓冲区，最终将当前的光标位置前进一位。
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                /*self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };*/
				self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
            }
        }
    }

	pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的 ASCII 码字节，也可以是换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 不包含在上述范围之内的字节
                _ => self.write_byte(0xfe),
            }

        }
    }


	fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

	fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}
// 实现 core::fmt::Write trait；
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// lazy_static! 的宏，定义了一个延迟初始化（lazily initialized）的静态变量；
// 这个变量的值将在第一次使用时计算，而非在编译时计算。
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}


// 实现 println! 宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}
// #[macro_export] 属性让整个包（crate）和基于它的包都能访问这个宏，而不仅限于定义它的模块（module）。
// 它还将把宏置于包的根模块（crate root）下，这意味着比如我们需要通过 use std::println 来导入这个宏，
// 而不是通过 std::macros::println。
#[macro_export]
macro_rules! println {
	// $crate 变量将在 std 包之外被解析为 std 包，保证整个宏在 std 包之外也可以使用。
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/*
_print 函数将占有静态变量 WRITER 的锁，并调用它的 write_fmt 方法。
这个方法是从名为 Write 的 trait 中获得的，所以需要导入这个 trait。
额外的 unwrap() 函数将在打印不成功的时候 panic；上面 write_str 总是返回 Ok，这种情况不应该发生。
*/
// doc(hidden) 属性，防止它在生成的文档中出现。
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) { //format_args! 宏将传入的参数搭建为一个 fmt::Arguments 类型，这个类型将被传入 _print 函数。
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
