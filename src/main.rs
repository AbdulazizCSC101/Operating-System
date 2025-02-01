#![no_std]

#![no_main]

// Import the bootloader’s entry point macro.
extern crate bootloader;
use bootloader::{entry_point, BootInfo};

mod framebuffer;
use core::fmt::{self, Write};

struct Writer;

impl Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Ok(())
    
        // Implement the actual writing logic here
    }
}

macro_rules! print {
    ($($arg:tt)*) => ($crate::write_fmt(format_args!($($arg)*)));
}

macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Implement the actual writing logic here
        Ok(())
    }
}

fn write_fmt(args: core::fmt::Arguments) {
    Writer.write_fmt(args).unwrap();
}

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    // Example output:
    print!("Hello, world!\nThis is a test.\n\\cBlue Blue Text\tIndented Text");
    
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}





    




