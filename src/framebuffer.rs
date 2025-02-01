// src/framebuffer.rs

#![no_std]
#![feature(alloc_error_handler)]
#![feature(alloc)]

extern crate alloc;
use core::fmt;
use alloc::{string::String, vec::Vec};
use core::fmt::Write;
use core::ops::{Deref, DerefMut};
use spin::Mutex;
use volatile::Volatile;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: u8,
}

impl DerefMut for ScreenChar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ascii_character
    }
}

impl Deref for ScreenChar {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.ascii_character
}
}

pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct FrameBufferWriter {
    column_position: usize,
    row_position: usize,
    pub color_code: u8,
    pub buffer: &'static mut Buffer,
}

impl FrameBufferWriter {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\t' => {
                for _ in 0..4 {
                    self.write_byte(b' ');
                }
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = self.row_position;
                let col = self.column_position;
                self.buffer.chars[row][col].write(*ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position + 1 < BUFFER_HEIGHT {
            self.row_position += 1;
        } else {
            self.scroll_up();
        }
        self.column_position = 0;
    }

    fn scroll_up(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
    }

    fn clear_row(&mut self, row: usize) {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(*ScreenChar {
                    ascii_character: b' ',
                    color_code: self.color_code,
                });
            }
        }

    pub fn color_from_name(name: &str) -> Option<u8> {
        match name.bytes().map(|b| b.to_ascii_lowercase()).collect::<Vec<_>>() {
            "black"       => Some(0x00),
            "blue"        => Some(0x01),
            "green"       => Some(0x02),
            "cyan"        => Some(0x03),
            "red"         => Some(0x04),
            "magenta"     => Some(0x05),
            "brown"       => Some(0x06),
            "light_gray"  => Some(0x07),
            "dark_gray"   => Some(0x08),
            "light_blue"  => Some(0x09),
            "light_green" => Some(0x0A),
            "light_cyan"  => Some(0x0B),
            "light_red"   => Some(0x0C),
            "pink"        => Some(0x0D),
            "yellow"      => Some(0x0E),
            "white"       => Some(0x0F),
            _ => None,
        }
    }
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER: Mutex<FrameBufferWriter> = Mutex::new(FrameBufferWriter {
        column_position: 0,
        row_position: 0,
        color_code: 0x0F, // Default white
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! fb_print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::framebuffer::WRITER.lock(), $($arg)*);
    });
}

#[macro_export]
macro_rules! fb_println {
    () => (fb_print!("\n"));
    ($fmt:expr) => (fb_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (fb_print!(concat!($fmt, "\n"), $($arg)*));
}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'n' => {
                            chars.next();
                            self.new_line();
                            continue;
                        }
                        't' => {
                            chars.next();
                            for _ in 0..4 {
                                self.write_byte(b' ');
                            }
                            continue;
                        }
                        'c' => {
                            chars.next(); // consume 'c'
                            let mut color_name = String::new();
                            while let Some(&c) = chars.peek() {
                                if c.is_whitespace() {
                                    break;
                                } else {
                                    color_name.push(c);
                                    chars.next();
                                }
                            }
                            if let Some(new_color) = FrameBufferWriter::color_from_name(&color_name) {
                                self.color_code = new_color;
                            }
                            continue;
                        }
                        _ => {}
                    }
                }
                self.write_byte(ch as u8);
            } else {
                self.write_byte(ch as u8);
            }
        }
        Ok(())
    }
}
