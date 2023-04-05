#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::{Color, TextOutput};
use uefi::table::boot::{AllocateType, MemoryType};

#[repr(C)]
struct MemoryMap 
{
    pub buffer_size: u64,
    pub buffer: *mut u8,
    pub map_size: u64,
    pub map_key: u64,
    pub descriptor_size: u32,
    pub descriptor_version: u32,
}

#[repr(C)]
struct MemoryDescriptor 
{
    pub ty: u32,
    pub pad: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

#[no_mangle]
pub extern "C" fn efi_main(image: uefi::Handle, system_table: SystemTable<Boot>) -> Status 
{
    let _ = system_table
        .stdout()
        .reset(false);
    let mut gop = system_table
        .boot_services()
        .locate_protocol::<GraphicsOutput>()
        .expect_success("Failed to locate GOP protocol");
    let mut text_out = system_table
        .console()
        .expect_success("Failed to open text console")
        .text_out();
    let _ = text_out
        .set_mode(0);
    let _ = text_out
        .set_color(Color::new(255, 255, 255), Color::new(0, 0, 0));
    let _ = writeln!(text_out, "Hello, world!");
    let memory_map_size = 
    {
        let mut memory_map: MemoryMap = MemoryMap 
        {
            buffer_size: 0,
            buffer: core::ptr::null_mut(),
            map_size: 0,
            map_key: 0,
            descriptor_size: 0,
            descriptor_version: 0,
        };
        let mut map_key = 0;
        let mut descriptor_size = 0;
        let mut descriptor_version = 0;
        let mut status = system_table.boot_services()
            .get_memory_map(&mut memory_map_size, memory_map.buffer,&mut map_key,&mut descriptor_size,&mut descriptor_version,);
 
        while status == Status::BUFFER_TOO_SMALL 
        {
            let buffer_size = memory_map.buffer_size + descriptor_size * 16;
            memory_map.buffer = system_table
                .boot_services()
                .allocate_pool(MemoryType::LOADER_DATA, buffer_size)
                .expect_success("Failed to allocate memory");
 
            memory_map.buffer_size = buffer_size;
 
            status = system_table.boot_services()
            .get_memory_map(&mut memory_map_size,memory_map.buffer,&mut map_key,&mut descriptor_size,&mut descriptor_version,);
        }
 
        if status != Status::SUCCESS 
        {
            return status;
        }
 
        memory_map.descriptor_size = descriptor_size;
        memory_map.descriptor_version = descriptor_version;
        memory_map.map_key = map_key;
 
        memory_map.map_size = memory_map_size;
 
        memory_map_size
    };
 
    let mut memory_map: MemoryMap = MemoryMap 
    {
        buffer_size: 0,
        buffer: core::ptr::null_mut(),
        map_size: 0,
        map_key: 0,
        descriptor_size: 0,
        descriptor_version: 0,
    };
 
    memory_map.buffer = system_table
        .boot_services()
        .allocate_pool(MemoryType::LOADER_DATA, memory_map_size)
        .expect_success("Failed to allocate memory");
 
    let mut map_key = 0;
    let mut descriptor_size = 0;
    let mut descriptor_version = 0;
    let mut status = system_table.boot_services().get_memory_map(
        &mut memory_map.map_size,
        memory_map.buffer,
        &mut map_key,
        &mut descriptor_size,
        &mut descriptor_version,
    );
 
    while status == Status::BUFFER_TOO_SMALL 
    {
        let buffer_size = memory_map.buffer_size + descriptor_size * 16;
        memory_map.buffer = system_table
            .boot_services()
            .allocate_pool(MemoryType::LOADER_DATA, buffer_size)
            .expect_success("Failed to allocate memory");
 
        memory_map.buffer_size = buffer_size;
 
        status = system_table.boot_services().get_memory_map(
            &mut memory_map.map_size,
            memory_map.buffer,
            &mut map_key,
            &mut descriptor_size,
            &mut descriptor_version,
        );
    }
 
    if status != Status::SUCCESS {
        return status;
    }
 
    let _ = system_table
        .boot_services()
        .exit_boot_services(image, memory_map.map_key)
        .expect_success("Failed to exit boot services");
    loop {}
}