#![feature(llvm_asm)]
#![feature(lang_items, start)]
#![no_std]
#![no_main]
#![feature(global_asm)]

use arch::ioport::IOPort;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr;
use model::Driver;
use payloads::payload;
use print;
use uart::i8250::I8250;

// Until we are done hacking on this, use our private copy.
// Plan to copy it back later.
global_asm!(include_str!("bootblock.S"));

//global_asm!(include_str!("init.S"));
fn poke(v: u32, a: u32) -> () {
    let y = a as *mut u32;
    unsafe {
        ptr::write_volatile(y, v);
    }
}

#[no_mangle]
pub extern "C" fn _start(fdt_address: usize) -> ! {
    let io = &mut IOPort;
    let post = &mut IOPort;
    let uart0 = &mut I8250::new(0x3f8, 0, io);
    uart0.init().unwrap();

    let mut count: u8 = 0;
    for _i in 0..1000000 {
        let mut p: [u8; 1] = [0; 1];
        for _j in 0..100000 {
            post.pread(&mut p, 0x3f8).unwrap();
        }
        count = count + 1;
        p[0] = count;
        post.pwrite(&p, 0x80).unwrap();

        uart0.pwrite(b"Welcome to oreboot\r\n", 0).unwrap();
    }

    let w = &mut print::WriteTo::new(uart0);

    let payload = &mut payload::StreamPayload { typ: payload::ftype::CBFS_TYPE_SELF, compression: payload::ctype::CBFS_COMPRESS_NONE, offset: 0, entry: 0x1000020 as usize, rom_len: 0 as usize, mem_len: 0 as usize, dtb: 0, rom: 0xff000000 };

    write!(w, "loading payload with fdt_address {}\r\n", fdt_address).unwrap();
    payload.load(w);
    if false {
        poke(0xfe, 0x100000);
    }
    write!(w, "Running payload\r\n").unwrap();
    payload.run(w);

    write!(w, "Unexpected return from payload\r\n").unwrap();
    arch::halt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Assume that uart0.init() has already been called before the panic.
    let io = &mut IOPort;
    let uart0 = &mut I8250::new(0x3f8, 0, io);
    let w = &mut print::WriteTo::new(uart0);
    // Printing in the panic handler is best-effort because we really don't want to invoke the panic
    // handler from inside itself.
    let _ = write!(w, "PANIC: {}\r\n", info);
    arch::halt()
}
