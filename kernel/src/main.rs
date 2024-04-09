#![feature(asm_const)]
#![no_main]
#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(format_args_nl)]

mod allocator;
mod bsp;
mod cpu;
mod fdt;
mod panic_wait;
mod print;
mod fs;

extern crate alloc;

use alloc::vec::Vec;
use allocator::MyAllocator;
use fs::cpio::CpioHandler;
use driver::mailbox;
use driver::uart;
use driver::addr_loader;

#[global_allocator]
static mut ALLOCATOR: MyAllocator = MyAllocator::new();

fn get_initrd_addr(name: &str, data: *const u8, len: usize) -> Option<u32> {
    if name == "linux,initrd-start" {
        unsafe { Some((*(data as *const u32)).swap_bytes()) }
    } else {
        None
    }
}

#[no_mangle]
fn kernel_init() -> ! {
    let mut dtb_addr = addr_loader::load_dtb_addr();
    // init uart
    uart::init_uart();
    uart::uart_write_str("Kernel started\r\n");
    // println!("DTB address: {:#x}", dtb_pos as u64);
    let dtb_parser = fdt::DtbParser::new(dtb_addr);

    let mut in_buf: [u8; 128] = [0; 128];
    // find initrd value by dtb traverse

    let mut initrd_start: *mut u8;
    if let Some(addr) = dtb_parser.traverse(get_initrd_addr) {
        initrd_start = addr as *mut u8;
        println!("Initrd start address: {:#x}", initrd_start as u64)
    } else {
        initrd_start = 0 as *mut u8;
    }

    let mut handler: CpioHandler = CpioHandler::new(initrd_start as *mut u8);
    loop {
        print!("meow>> ");
        let inp = alloc::string::String::from(uart::getline(&mut in_buf, true));
        let cmd = inp.trim().split(' ').collect::<Vec<&str>>();
        // split the input string
        match cmd[0] {
            "hello" => println!("Hello World!"),
            "help" => println!("help    : print this help menu\r\nhello   : print Hello World!\r\nreboot  : reboot the device"),
            "reboot" => {
                uart::reboot();
                break;
            }
            "ls" => {
                for file in handler.get_files() {
                    println!("{}", file.get_name());
                }
            }
            "cat" => {
                if cmd.len() < 2 {
                    println!("Usage: cat <file>");
                    continue;
                }
                let mut file = handler.get_files().find(|f| f.get_name() == cmd[1]);
                if let Some(mut f) = file {
                    println!("File size: {}", f.get_size());
                    loop {
                        let data = f.read(32);
                        if data.len() == 0 {
                            break;
                        }
                        for i in data {
                            unsafe {
                                uart::write_u8(*i);
                            }
                        }
                    }
                } else {
                    println!("File not found");
                }
            }
            "dtb" => {
                unsafe { println!("Start address: {:#x}", dtb_addr as u64) };
                println!("dtb load address: {:#x}", dtb_addr as u64);
                println!("dtb pos: {:#x}", unsafe {*(dtb_addr)});
                dtb_parser.parse_struct_block();
            }
            "mailbox" => {
                println!("Revision: {:#x}", mailbox::get_board_revisioin());
                let (base, size) = mailbox::get_arm_memory();
                println!("ARM memory base address: {:#x}", base);
                println!("ARM memory size: {:#x}", size);
            }
            "exec" => {

            }
            "" => {}
            _ => {
                println!("Command not found\r\n");
            }
        }
    }
    
    panic!()
}
