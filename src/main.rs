#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use uefi::prelude::*;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::proto::console::text::Key;
use core::fmt::Write;
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;
use alloc::string::String;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
  
    {
        let bs = system_table.boot_services();
        let heap_size = 1024 * 1024;
        let heap_pages = (heap_size + 4095) / 4096;
        
        let heap_addr = bs.allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            heap_pages,
        ).unwrap();
        
        unsafe {
            ALLOCATOR.lock().init(heap_addr as *mut u8, heap_size);
        }
    }
    
    system_table.stdout().clear().unwrap();
    
    writeln!(system_table.stdout(), "<ReeOS>").unwrap();
    writeln!(system_table.stdout(), "Type 'help' for commands").unwrap();
    writeln!(system_table.stdout()).unwrap();
    
    let mut input_buffer = String::new();
    
    write!(system_table.stdout(), "> ").unwrap();
    
    loop {
        let key = system_table.stdin().read_key().unwrap();
        
        if let Some(key) = key {
            match key {
                Key::Printable(c16) => {
                    let u16_val: u16 = c16.into();
                    
                    match u16_val {
                        0x000D => { 
                            writeln!(system_table.stdout()).unwrap();
                            
                            match input_buffer.trim() {
                                "help" => {
                                    writeln!(system_table.stdout(), "help  - show commands").unwrap();
                                    writeln!(system_table.stdout(), "clear - clear screen").unwrap();
                                    writeln!(system_table.stdout(), "echo  - print text").unwrap();
                                    
                                }
                                "clear" => {
                                    system_table.stdout().clear().unwrap();
                                }
                                
                                cmd if cmd.starts_with("echo ") => {
                                    writeln!(system_table.stdout(), "{}", &cmd[5..]).unwrap();
                                }
                                "" => {}
                                cmd => {
                                    writeln!(system_table.stdout(), "Unknown: '{}'", cmd).unwrap();
                                }
                            }
                            
                            input_buffer.clear();
                            write!(system_table.stdout(), "> ").unwrap();
                        }
                        0x0008 => {
                            if !input_buffer.is_empty() {
                                input_buffer.pop();
                                write!(system_table.stdout(), "\x08 \x08").unwrap();
                            }
                        }
                        _ => {
                           
                            if u16_val >= 0x20 && u16_val < 0x7F {
                                if let Some(c) = char::from_u32(u16_val as u32) {
                                    input_buffer.push(c);
                                    write!(system_table.stdout(), "{}", c).unwrap();
                                }
                            }
                           
                        }
                    }
                }
                Key::Special(_) => {
                   
                }
            }
        }
    }
}
