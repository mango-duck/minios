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
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// 将让这个 Writer 类型将字符写入屏幕的最后一行，并在一行写满或接收到换行符 \n 的时候，将所有的字符向上位移一行。
/*
我们存入一个 VGA 字符缓冲区的可变借用到buffer变量中。
需要注意的是，这里我们对借用使用显式生命周期（explicit lifetime），
告诉编译器这个借用在何时有效：
我们使用 'static 生命周期（’static lifetime），意味着这个借用应该在整个程序的运行期间有效；
这对一个全局有效的 VGA 字符缓冲区来说，是非常合理的。
*/
pub struct Writer {
    column_position: usize,  // column_position 变量将跟踪光标在最后一行的位置。当前字符的前景和背景色将由 color_code 变量指定；
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

/*
如果这个字节是一个换行符（line feed）字节 \n，我们的 Writer 不应该打印新字符，相反，它将调用我们稍后会实现的 new_line 方法；其它的字节应该将在 match 语句的第二个分支中被打印到屏幕上。
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
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {/* TODO */}
}

impl Writer {
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
}

// 这个函数首先创建一个指向 0xb8000 地址VGA缓冲区的 Writer。
pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
		/*
			首先，我们把整数 0xb8000 强制转换为一个可变的裸指针（raw pointer）；
			之后，通过运算符*，我们将这个裸指针解引用；
			最后，我们再通过 &mut，再次获得它的可变借用。
			这些转换需要 unsafe 语句块（unsafe block），因为编译器并不能保证这个裸指针是有效的。
		*/
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
}


