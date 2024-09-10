#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
// reexport_test_harness_main属性来将生成的函数的名称更改为与main不同的名称。
#![reexport_test_harness_main = "test_main"]


use core::panic::PanicInfo;

mod vga_buffer;


#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("Hello World{}", "!");

	#[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
    loop {}
}


// 参数类型 &[&dyn Fn()] 是Fn() trait的 trait object 引用的一个 slice。
// 它基本上可以被看做一个可以像函数一样被调用的类型的引用列表。
// #[cfg(test)]属性保证它只会出现在测试中。
#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
	
	exit_qemu(QemuExitCode::Success); // 执行退出操作
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
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
