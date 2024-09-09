#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;


#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("Hello World{}", "!");
	println!("Hello World{}", "!");
	println!("Hello World{}", "!");
	assert_eq!(0, 1);
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
    loop {}
}
