#![feature(lang_items, asm, core_intrinsics)]
#![no_std]

pub mod gpio;
pub mod uart;

#[no_mangle]
pub extern "C" fn main() {
    let u = uart::Uart::new();
    u.puts("Hello Rust Kernel world!\n");

    loop {
        u.send(u.getc() as u32)
    }
}

#[lang = "panic_fmt"]
extern "C" fn panic_fmt() -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    main()
}
