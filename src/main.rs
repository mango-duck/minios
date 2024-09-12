#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(minios::test_runner)]
// reexport_test_harness_main属性来将生成的函数的名称更改为与main不同的名称。
#![reexport_test_harness_main = "test_main"]

use minios::println;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("Hello World{}", "!");

	#[cfg(test)]
    test_main();

    loop {}
}

#[cfg(not(test))] 
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	minios::test_panic_handler(info)
}

// 添加一个测试函数
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
