#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use uefi::prelude::*;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::proto::console::text::Key;
use uefi::proto::console::gop::GraphicsOutput;
use core::fmt::Write;
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

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

struct PackageManager {
    installed: BTreeMap<String, String>,
    available: Vec<String>,
    initialized: bool,
}

impl PackageManager {
    fn new() -> Self {
        Self {
            installed: BTreeMap::new(),
            available: Vec::new(),
            initialized: false,
        }
    }
    
    fn init(&mut self) {
        self.available.push(String::from("base"));
        self.available.push(String::from("dev"));
        self.available.push(String::from("game"));
        self.initialized = true;
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    fn install(&mut self, package: &str) -> bool {
        if !self.initialized {
            return false;
        }
        
        if self.available.contains(&String::from(package)) {
            if !self.installed.contains_key(package) {
                self.installed.insert(String::from(package), String::from("1.0.0"));
                return true;
            }
        }
        false
    }
    
    fn list_installed(&self) -> Vec<String> {
        if !self.initialized {
            return Vec::new();
        }
        self.installed.keys().cloned().collect()
    }
    
    fn list_available(&self) -> Vec<String> {
        if !self.initialized {
            return Vec::new();
        }
        self.available.clone()
    }
}

fn show_progress_bar(stdout: &mut uefi::proto::console::text::Output, percent: usize) {
    write!(stdout, "\r[").unwrap();
    let filled = percent / 5;
    let empty = 20 - filled;
    for _ in 0..filled {
        write!(stdout, "=").unwrap();
    }
    for _ in 0..empty {
        write!(stdout, " ").unwrap();
    }
    write!(stdout, "] {}%", percent).unwrap();
}

fn simulate_init(stdout: &mut uefi::proto::console::text::Output) {
    writeln!(stdout, "initializing rpm database...").unwrap();
    for i in 0..=100 {
        show_progress_bar(stdout, i);
        for _ in 0..20000  {
            core::hint::spin_loop();
        }
    }
    writeln!(stdout).unwrap();
    writeln!(stdout, "Done!").unwrap();
}

fn simulate_download(stdout: &mut uefi::proto::console::text::Output, package: &str) {
    writeln!(stdout, "downloading {}...", package).unwrap();
    for i in 0..=100 {
        show_progress_bar(stdout, i);
        for _ in 0..50000 {
            core::hint::spin_loop();
        }
    }
    writeln!(stdout).unwrap();
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
    
    writeln!(system_table.stdout(), "========================================").unwrap();
    writeln!(system_table.stdout(), "         <ReeOS> ").unwrap();
    writeln!(system_table.stdout(), "========================================").unwrap();
    writeln!(system_table.stdout()).unwrap();
    writeln!(system_table.stdout(), "Type 'help' for commands").unwrap();
    writeln!(system_table.stdout()).unwrap();
    
    let mut pkg_manager = PackageManager::new();
    let mut files: Vec<(String, String)> = Vec::new();
    
    files.push((String::from("readme.txt"), String::from("Welcome to ReeOS!")));
    files.push((String::from("hello.txt"), String::from("Hello World!")));
    
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
                            
                            let parts: Vec<&str> = input_buffer.split_whitespace().collect();
                            
                            match parts.as_slice() {
                                ["help"] => {
                                    writeln!(system_table.stdout()).unwrap();
                                    writeln!(system_table.stdout(), "=== system commands ===").unwrap();
                                    writeln!(system_table.stdout(), "  help, clear, echo, ls, touch, cat, rm").unwrap();
                                    writeln!(system_table.stdout()).unwrap();
                                    writeln!(system_table.stdout(), "=== ReeOS Package Manager(rpm) ===").unwrap();
                                    writeln!(system_table.stdout(), "  rpm init            - Initialize package database").unwrap();
                                    writeln!(system_table.stdout(), "  rpm install <pkg>   - Install a package").unwrap();
                                    writeln!(system_table.stdout(), "  rpm remove <pkg>    - Remove a package").unwrap();
                                    writeln!(system_table.stdout(), "  rpm list            - List installed packages").unwrap();
                                    writeln!(system_table.stdout(), "  rpm available       - Show available packages").unwrap();
                                    writeln!(system_table.stdout()).unwrap();
                                    writeln!(system_table.stdout(), "  halt                - Stop CPU").unwrap();
                                }
                                ["info"] =>{
                                    writeln!(system_table.stdout(), "Version OS: 1.0.6").unwrap();
                                    writeln!(system_table.stdout(), "Version UEFI").unwrap();
                                    writeln!(system_table.stdout(), "ReeOS Package Manager(rpm)").unwrap();
                                    writeln!(system_table.stdout(), "-install additional commands").unwrap();
                                }
                                ["clear"] => {
                                    system_table.stdout().clear().unwrap();
                                    writeln!(system_table.stdout(), "<ReeOS> - Terminal cleared").unwrap();
                                }
                                ["halt"] => {
                                    writeln!(system_table.stdout(), "CPU halted").unwrap();
                                    loop {
                                        unsafe { core::arch::asm!("hlt") };
                                    }
                                }
                                ["echo", rest @ ..] => {
                                    writeln!(system_table.stdout(), "{}", rest.join(" ")).unwrap();
                                }
                                ["ls"] => {
                                    if files.is_empty() {
                                        writeln!(system_table.stdout(), "(empty)").unwrap();
                                    } else {
                                        writeln!(system_table.stdout(), "Files:").unwrap();
                                        for (name, _) in &files {
                                            writeln!(system_table.stdout(), "  - {}", name).unwrap();
                                        }
                                    }
                                }
                                ["cat", filename] => {
                                    let mut found = false;
                                    for (name, content) in &files {
                                        if name == *filename {
                                            writeln!(system_table.stdout(), "{}", content).unwrap();
                                            found = true;
                                            break;
                                        }
                                    }
                                    if !found {
                                        writeln!(system_table.stdout(), "File not found: {}", filename).unwrap();
                                    }
                                }
                                ["touch", filename, content @ ..] => {
                                    let text = content.join(" ");
                                    files.push((String::from(*filename), text));
                                    writeln!(system_table.stdout(), "Created: {}", filename).unwrap();
                                }
                                ["rm", filename] => {
                                    if let Some(pos) = files.iter().position(|(n, _)| n == *filename) {
                                        files.remove(pos);
                                        writeln!(system_table.stdout(), "Deleted: {}", filename).unwrap();
                                    } else {
                                        writeln!(system_table.stdout(), "File not found: {}", filename).unwrap();
                                    }
                                }
                                ["rpm", "init"] => {
                                    if pkg_manager.is_initialized() {
                                        writeln!(system_table.stdout(), "RPM already initialized!").unwrap();
                                    } else {
                                        simulate_init(system_table.stdout());
                                        pkg_manager.init();
                                    }
                                }
                                ["rpm", "install", package] => {
                                    if !pkg_manager.is_initialized() {
                                        writeln!(system_table.stdout(), "error: RPM not initialized! Run 'rpm init' first.").unwrap();
                                    } else {
                                        simulate_download(system_table.stdout(), package);
                                        if pkg_manager.install(package) {
                                            writeln!(system_table.stdout(), "succes: Installed {}", package).unwrap();
                                        } else {
                                            writeln!(system_table.stdout(), "error: Package not found or already installed").unwrap();
                                        }
                                    }
                                }
                                ["rpm", "remove", package] => {
                                    if !pkg_manager.is_initialized() {
                                        writeln!(system_table.stdout(), "error: RPM not initialized! Run 'rpm init' first.").unwrap();
                                    } else {
                                        if pkg_manager.installed.remove(&String::from(*package)).is_some() {
                                            writeln!(system_table.stdout(), "SUCCESS: Removed {}", package).unwrap();
                                        } else {
                                            writeln!(system_table.stdout(), "error: Package not installed").unwrap();
                                        }
                                    }
                                }
                                ["rpm", "list"] => {
                                    if !pkg_manager.is_initialized() {
                                        writeln!(system_table.stdout(), "error: RPM not initialized! Run 'rpm init' first.").unwrap();
                                    } else {
                                        let installed = pkg_manager.list_installed();
                                        if installed.is_empty() {
                                            writeln!(system_table.stdout(), "No packages installed").unwrap();
                                        } else {
                                            writeln!(system_table.stdout(), "Installed packages:").unwrap();
                                            for pkg in installed {
                                                writeln!(system_table.stdout(), "  - {} v1.0.0", pkg).unwrap();
                                            }
                                        }
                                    }
                                }
                                ["rpm", "available"] => {
                                    if !pkg_manager.is_initialized() {
                                        writeln!(system_table.stdout(), "error: RPM not initialized! Run 'rpm init' first.").unwrap();
                                    } else {
                                        writeln!(system_table.stdout(), "Available packages:").unwrap();
                                        for pkg in pkg_manager.list_available() {
                                            let status = if pkg_manager.installed.contains_key(&pkg) {
                                                "[installed]"
                                            } else {
                                                ""
                                            };
                                            writeln!(system_table.stdout(), "  - {} {}", pkg, status).unwrap();
                                        }
                                    }
                                }
                                [""] => {}
                                [cmd, ..] => {
                                    writeln!(system_table.stdout(), "Unknown command: '{}'", cmd).unwrap();
                                    writeln!(system_table.stdout(), "Type 'help' for available commands").unwrap();
                                }
                                [] => {}
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
                Key::Special(_) => {}
            }
        }
    }
}
