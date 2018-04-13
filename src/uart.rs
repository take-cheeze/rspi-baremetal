// raspi2 and raspi3 have peripheral base address 0x3F000000,
// but raspi1 has peripheral base address 0x20000000. Ensure
// you are using the correct peripheral address for your
// hardware.
const MMIO_BASE: u32 = 0x3F000000;

unsafe fn mmio_write(reg: u32, val: u32) {
    ::core::ptr::write_volatile(reg as *mut u32, val)
}

unsafe fn mmio_read(reg: u32) -> u32 {
    ::core::ptr::read_volatile(reg as *const u32)
}

const UART0_DR: u32 = MMIO_BASE + 0x00201000;
const UART0_FR: u32 = MMIO_BASE + 0x00201018;
const UART0_IBRD: u32 = MMIO_BASE + 0x00201024;
const UART0_FBRD: u32 = MMIO_BASE + 0x00201028;
const UART0_LCRH: u32 = MMIO_BASE + 0x0020102C;
const UART0_CR: u32 = MMIO_BASE + 0x00201030;
// const UART0_IMSC:u32 = MMIO_BASE+0x00201038;
const UART0_ICR: u32 = MMIO_BASE + 0x00201044;

#[repr(align(16))]
struct MBox {
    mbox: [u32; 36],
}

static MBOX: MBox = MBox { mbox: [0; 36] };

pub struct Uart {}

const MBOX_REQUEST: u32 = 0;
const MBOX_TAG_SETCLKRATE: u32 = 0x38002;
const MBOX_TAG_LAST: u32 = 0;

const VIDEOCORE_MBOX: u32 = MMIO_BASE + 0x0000B880;
const MBOX_READ: u32 = VIDEOCORE_MBOX + 0x0;
// const MBOX_POLL: u32   =     VIDEOCORE_MBOX+0x10;
// const MBOX_SENDER: u32  =   VIDEOCORE_MBOX+0x14;
const MBOX_STATUS: u32 = VIDEOCORE_MBOX + 0x18;
// const MBOX_CONFIG: u32    = VIDEOCORE_MBOX+0x1C;
const MBOX_WRITE: u32 = VIDEOCORE_MBOX + 0x20;
const MBOX_RESPONSE: u32 = 0x80000000;
const MBOX_FULL: u32 = 0x80000000;
const MBOX_EMPTY: u32 = 0x40000000;

// const MBOX_CH_POWER: u8 = 0;
// const MBOX_CH_FB: u8 = 1;
// const MBOX_CH_VUART: u8 = 2;
// const MBOX_CH_VCHIQ: u8 = 3;
// const MBOX_CH_LEDS: u8 = 4;
// const MBOX_CH_BTNS: u8 = 5;
// const MBOX_CH_TOUCH: u8 = 6;
// const MBOX_CH_COUNT: u8 = 7;
const MBOX_CH_PROP: u8 = 8;

// const GPFSEL0: u32 = MMIO_BASE+0x00200000;
const GPFSEL1: u32 = MMIO_BASE + 0x00200004;
/*
const GPFSEL2: u32 = MMIO_BASE+0x00200008;
const GPFSEL3: u32 = MMIO_BASE+0x0020000C;
const GPFSEL4: u32 = MMIO_BASE+0x00200010;
const GPFSEL5: u32 = MMIO_BASE+0x00200014;
const GPSET0: u32 = MMIO_BASE+0x0020001C;
const GPSET1: u32 = MMIO_BASE+0x00200020;
const GPCLR0: u32 = MMIO_BASE+0x00200028;
const GPLEV0: u32 = MMIO_BASE+0x00200034;
const GPLEV1: u32 = MMIO_BASE+0x00200038;
const GPEDS0: u32 = MMIO_BASE+0x00200040;
const GPEDS1: u32 = MMIO_BASE+0x00200044;
const GPHEN0: u32 = MMIO_BASE+0x00200064;
const GPHEN1: u32 = MMIO_BASE+0x00200068;
 */
const GPPUD: u32 = MMIO_BASE + 0x00200094;
const GPPUDCLK0: u32 = MMIO_BASE + 0x00200098;
// const GPPUDCLK1: u32 = MMIO_BASE+0x0020009C;

unsafe fn mbox_call(ch: u8) -> bool {
    /* wait until we can write to the mailbox */
    while (mmio_read(MBOX_STATUS) & MBOX_FULL) > 0 {
        asm!("nop")
    }

    /* write the address of our message to the mailbox with channel identifier */
    mmio_write(
        MBOX_WRITE,
        (((&MBOX as *const MBox as u32) & !0xF) | ((ch & 0xF) as u32)) as u32,
    );

    /* now wait for the response */
    loop {
        /* is there a response? */
        while (mmio_read(MBOX_STATUS) & MBOX_EMPTY) > 0 {
            asm!("nop");
        }
        let r = mmio_read(MBOX_READ);
        /* is it a response to our message? */
        if ((r & 0xF) as u8) == ch && (r & !0xF) == (&MBOX as *const MBox as u32) {
            /* is it a valid successful response? */
            return MBOX.mbox[1] == MBOX_RESPONSE;
        }
    }
}

impl Uart {
    /**
     * Set baud rate and characteristics (115200 8N1) and map to GPIO
     */
    pub fn new() -> Uart {
        // return Uart{};
        unsafe {
            /* initialize UART */
            mmio_write(UART0_CR, 0); // turn off UART0

            // set up clock for consistent divisor values
            let mut m = MBOX.mbox;
            m[0] = 8 * 4;
            m[1] = MBOX_REQUEST;
            m[2] = MBOX_TAG_SETCLKRATE; // set clock rate
            m[3] = 12;
            m[4] = 8;
            m[5] = 2; // UART clock
            m[6] = 4000000; // 4Mhz
            m[7] = MBOX_TAG_LAST;
            mbox_call(MBOX_CH_PROP);

          /*
            // map UART0 to GPIO pins
            let mut r = mmio_read(GPFSEL1);
            r &= !((7 << 12) | (7 << 15)); // gpio14, gpio15
            r |= (4 << 12) | (4 << 15); // alt0
            mmio_write(GPFSEL1, r);
            mmio_write(GPPUD, 0); // enable pins 14 and 15
            for _ in 0..150 { asm!("nop") }
            mmio_write(GPPUDCLK0, (1 << 14) | (1 << 15));
            for _ in 0..150 { asm!("nop") }
            mmio_write(GPPUDCLK0, 0); // flush GPIO setup
*/

            mmio_write(UART0_ICR, 0x7FF); // clear interrupts
            mmio_write(UART0_IBRD, 2); // 115200 baud
            mmio_write(UART0_FBRD, 0xB);
            mmio_write(UART0_LCRH, 0b11 << 5); // 8n1
            mmio_write(UART0_CR, 0x301); // enable Tx, Rx, FIFO

            Uart {}
        }
    }

    /**
     * Send a character
     */
    pub fn send(&self, c: u32) {
        unsafe {
            /* wait until we can send */
            while (mmio_read(UART0_FR) & (1 << 5)) > 0 {
                asm!("nop")
            }

          // convert newline to carrige return + newline
            /* write the character to the buffer */
          mmio_write(UART0_DR, match c as u8 as char {
            '\n' => '\r' as u32,
            _ => c as u32
          });
        }
    }

    /**
     * Receive a character
     */
    pub fn getc(&self) -> char {
        unsafe {
            /* wait until something is in the buffer */
            while (mmio_read(UART0_FR) & (1 << 4)) > 0 {
                asm!("nop")
            }
            /* read it and return */
            let r = mmio_read(UART0_DR) as u8;
            /* convert carrige return to newline */
            match r as char {
                '\r' => '\n',
                c => c,
            }
        }
    }

    /**
     * Display a string
     */
    pub fn puts(&self, s: &str) {
        for c in s.chars() {
            self.send(c as u32)
        }
    }

    /**
     * Display a binary value in hexadecimal
     */
    pub fn hex(&self, d: u32) {
        for c in 7..=0 {
            // get highest tetrad
            let mut n = (d >> (c * 4)) & 0xF;
            // 0-9 => '0'-'9', 10-15 => 'A'-'F'
            n += if n > 9 { 0x37 } else { 0x30 };
            self.send(n);
        }
    }
}
