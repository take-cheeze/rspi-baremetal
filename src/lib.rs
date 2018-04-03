#![feature(lang_items, asm, core_intrinsics)]
#![no_std]

pub mod gpio;
pub mod uart;

#[no_mangle]
pub extern "C" fn _start() {
    main()
}

#[no_mangle]
pub extern "C" fn main() {
  let u = uart::Uart::new();
    u.puts("Hello Rust Kernel world!");

    loop {
        u.send(u.getc() as u32)
    }

    gpio::init_led();

    loop {
        gpio::led_off();
        for _ in 1..500000 {
            unsafe {
                asm!("");
            }
        }

        gpio::led_on();
        for _ in 1..500000 {
            unsafe {
                asm!("");
            }
        }
    }
}

#[lang = "panic_fmt"]
extern "C" fn panic_fmt() -> ! {
    loop {}
}
