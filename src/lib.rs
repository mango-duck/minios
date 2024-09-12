#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode { // 指定退出状态码
    Success = 0x10,
    Failed = 0x11,
}
/*
	该函数在 0xf4 处创建一个新的端口，该端口同时也是 isa-debug-exit 设备的 iobase 。然后它会向端口写入传递的退出代码。
*/
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
	//两个操作都是 unsafe 的，因为I/O端口的写入操作通常会导致一些不可预知的行为。
    unsafe { 
        let mut port = Port::new(0xf4); 
        port.write(exit_code as u32); //使用 u32 来传递数据, Cargo.toml中iosize 指定为4字节
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	test_panic_handler(info)
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}
