#![feature(lang_items, asm, core_intrinsics)]
#![no_std]

use core::intrinsics::volatile_load;
use core::intrinsics::volatile_store;

pub mod gpio;

// raspi2 and raspi3 have peripheral base address 0x3F000000,
// but raspi1 has peripheral base address 0x20000000. Ensure
// you are using the correct peripheral address for your
// hardware.
const UART_DR: u32 = 0x3F201000;
const UART_FR: u32 = 0x3F201018;

fn mmio_write(reg: u32, val: u32) {
    unsafe { volatile_store(reg as *mut u32, val) }
}

fn mmio_read(reg: u32) -> u32 {
    unsafe { volatile_load(reg as *const u32) }
}

fn transmit_fifo_full() -> bool {
    mmio_read(UART_FR) & (1 << 5) > 0
}

fn receive_fifo_empty() -> bool {
    mmio_read(UART_FR) & (1 << 4) > 0
}

fn writec(c: u8) {
    while transmit_fifo_full() {}
    mmio_write(UART_DR, c as u32);
}

fn getc() -> u8 {
    while receive_fifo_empty() {}
    mmio_read(UART_DR) as u8
}

fn write(msg: &str) {
    for c in msg.chars() {
        writec(c as u8)
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    main()
}

#[no_mangle]
pub extern "C" fn main() {
    use gpio::*;

    write("Hello Rust Kernel world!");

    loop {
        writec(getc())
    }

    let gpio = GPIO_BASE as *const u32;
    let init = unsafe { gpio.offset(LED_GPFSEL) as *mut u32 };
    let led_on = unsafe { gpio.offset(LED_GPSET) as *mut u32 };
    let led_off = unsafe { gpio.offset(LED_GPCLR) as *mut u32 };

    unsafe {
        volatile_store(init, *(init) | 1 << LED_GPFBIT);
    }

    unsafe {
        volatile_store(led_on, 1 << LED_GPIO_BIT);
    }
    loop {
        unsafe {
            volatile_store(led_off, 1 << LED_GPIO_BIT);
        }
        for _ in 1..500000 {
            unsafe {
                asm!("");
            }
        }

        unsafe {
            volatile_store(led_on, 1 << LED_GPIO_BIT);
        }
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
