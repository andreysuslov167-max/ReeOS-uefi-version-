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
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

mod ramfs {
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::vec::Vec;

    #[derive(Clone)]
    pub struct File {
        pub name: String,
        pub content: Vec<u8>,
        pub is_dir: bool,
        pub children: BTreeMap<String, File>,
    }

    impl File {
        pub fn new_file(name: &str) -> Self {
            Self {
                name: String::from(name),
                content: Vec::new(),
                is_dir: false,
                children: BTreeMap::new(),
            }
        }

        pub fn new_dir(name: &str) -> Self {
            Self {
                name: String::from(name),
                content: Vec::new(),
                is_dir: true,
                children: BTreeMap::new(),
            }
        }
    }

    pub struct RamFS {
        root: File,
    }

    impl RamFS {
        pub fn new() -> Self {
            let mut root = File::new_dir("/");
            
            let mut readme = File::new_file("README.TXT");
            readme.content = b"Welcome to ReeOS!\nThis is a RAM filesystem.\n".to_vec();
            root.children.insert(String::from("README.TXT"), readme);
            
            let mut hello = File::new_file("HELLO.TXT");
            hello.content = b"Hello, World!\n".to_vec();
            root.children.insert(String::from("HELLO.TXT"), hello);
            
            let docs = File::new_dir("DOCS");
            root.children.insert(String::from("DOCS"), docs);
            
            Self { root }
        }

        pub fn list_dir(&self, path: &str) -> Vec<String> {
            let mut current = &self.root;
            
            if path != "/" {
                for part in path.split('/').filter(|p| !p.is_empty()) {
                    if let Some(child) = current.children.get(part) {
                        current = child;
                    } else {
                        return Vec::new();
                    }
                }
            }
            
            let mut result: Vec<String> = current.children.keys().cloned().collect();
            result.sort();
            result
        }

        pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
            let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
            if parts.is_empty() {
                return None;
            }
            
            let filename = parts.last().unwrap();
            let dir_path: Vec<&str> = if parts.len() > 1 {
                parts[..parts.len()-1].to_vec()
            } else {
                Vec::new()
            };
            
            let mut current = &self.root;
            for part in dir_path {
                if let Some(child) = current.children.get(part) {
                    current = child;
                } else {
                    return None;
                }
            }
            
            current.children.get(*filename).map(|f| f.content.clone())
        }

        pub fn create_file(&mut self, path: &str, content: Vec<u8>) -> Result<(), &'static str> {
            let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
            if parts.is_empty() {
                return Err("Invalid path");
            }
            
            let filename = parts.last().unwrap();
            let dir_path: Vec<&str> = if parts.len() > 1 {
                parts[..parts.len()-1].to_vec()
            } else {
                Vec::new()
            };
            
            let mut current = &mut self.root;
            for part in dir_path {
                if let Some(child) = current.children.get_mut(part) {
                    current = child;
                } else {
                    return Err("Directory not found");
                }
            }
            
            let mut file = File::new_file(filename);
            file.content = content;
            current.children.insert(String::from(*filename), file);
            Ok(())
        }

        pub fn delete(&mut self, path: &str) -> Result<(), &'static str> {
            let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
            if parts.is_empty() {
                return Err("Invalid path");
            }
            
            let filename = parts.last().unwrap();
            let dir_path: Vec<&str> = if parts.len() > 1 {
                parts[..parts.len()-1].to_vec()
            } else {
                Vec::new()
            };
            
            let mut current = &mut self.root;
            for part in dir_path {
                if let Some(child) = current.children.get_mut(part) {
                    current = child;
                } else {
                    return Err("Directory not found");
                }
            }
            
            current.children.remove(&String::from(*filename));
            Ok(())
        }
    }
}

use ramfs::RamFS;

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
    
    let mut fs = RamFS::new();
    
    system_table.stdout().clear().unwrap();
    writeln!(system_table.stdout(), "<ReeOS> with RAM Filesystem").unwrap();
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
                            
                            let parts: Vec<&str> = input_buffer.split_whitespace().collect();
                            
                            match parts.as_slice() {
                                ["help"] => {
                                    writeln!(system_table.stdout(), "Available commands:").unwrap();
                                    writeln!(system_table.stdout(), "  help                - show this help").unwrap();
                                    writeln!(system_table.stdout(), "  clear               - clear screen").unwrap();
                                    writeln!(system_table.stdout(), "  echo <text>         - print text").unwrap();
                                    writeln!(system_table.stdout(), "  ls [path]           - list directory").unwrap();
                                    writeln!(system_table.stdout(), "  cat <file>          - show file content").unwrap();
                                    writeln!(system_table.stdout(), "  touch <file> <text> - create file").unwrap();
                                    writeln!(system_table.stdout(), "  rm <file>           - delete file").unwrap();
                                    writeln!(system_table.stdout(), "  halt                - stop CPU").unwrap();
                                }
                                ["clear"] => {
                                    system_table.stdout().clear().unwrap();
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
                                    let files = fs.list_dir("/");
                                    if files.is_empty() {
                                        writeln!(system_table.stdout(), "(empty)").unwrap();
                                    } else {
                                        for file in files {
                                            writeln!(system_table.stdout(), "  {}", file).unwrap();
                                        }
                                    }
                                }
                                ["ls", path] => {
                                    let files = fs.list_dir(path);
                                    if files.is_empty() {
                                        writeln!(system_table.stdout(), "(empty)").unwrap();
                                    } else {
                                        for file in files {
                                            writeln!(system_table.stdout(), "  {}", file).unwrap();
                                        }
                                    }
                                }
                                ["cat", filename] => {
                                    match fs.read_file(filename) {
                                        Some(content) => {
                                            if let Ok(text) = core::str::from_utf8(&content) {
                                                write!(system_table.stdout(), "{}", text).unwrap();
                                            } else {
                                                writeln!(system_table.stdout(), "(binary file)").unwrap();
                                            }
                                        }
                                        None => {
                                            writeln!(system_table.stdout(), "File not found: {}", filename).unwrap();
                                        }
                                    }
                                }
                                ["touch", filename, content @ ..] => {
                                    let text = content.join(" ").as_bytes().to_vec();
                                    match fs.create_file(filename, text) {
                                        Ok(()) => {
                                            writeln!(system_table.stdout(), "Created: {}", filename).unwrap();
                                        }
                                        Err(e) => {
                                            writeln!(system_table.stdout(), "Error: {}", e).unwrap();
                                        }
                                    }
                                }
                                ["rm", filename] => {
                                    match fs.delete(filename) {
                                        Ok(()) => {
                                            writeln!(system_table.stdout(), "Deleted: {}", filename).unwrap();
                                        }
                                        Err(e) => {
                                            writeln!(system_table.stdout(), "Error: {}", e).unwrap();
                                        }
                                    }
                                }
                                [""] => {}
                                [cmd, ..] => {
                                    writeln!(system_table.stdout(), "Unknown command: '{}'", cmd).unwrap();
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