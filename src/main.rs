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
        let mut descriptor_buffer: *mut u8 = core::ptr::null_mut();
        let mut descriptor_buffer_size = 0;
        let mut descriptor_version = 0;
        loop 
        {
            let status = system_table
                .boot_services()
                .get_memory_map( &mut memory_map_size,memory_map.buffer, &mut map_key, &mut memory_map.descriptor_size, &mut memory_map.descriptor_version,);
            match status 
            {
                Status::SUCCESS => 
                {
                    descriptor_buffer_size = memory_map_size;
                    descriptor_buffer = system_table
                        .boot_services()
                        .allocate_pool(MemoryType::LOADER_DATA, descriptor_buffer_size)
                        .expect_success("Failed to allocate memory for descriptor buffer");
                    break;
                }
                Status::BUFFER_TOO_SMALL => 
                {
                    memory_map.buffer_size = memory_map_size;
                    memory_map.buffer = system_table.boot_services()
                        .allocate_pool(MemoryType::LOADER_DATA, memory_map.buffer_size)
                        .expect_success("Failed to allocate memory for memory map buffer");
                }
                _ => panic!("Failed to get memory map: {:?}", status),
            }
        }
 
        let mut descriptors: Vec<MemoryDescriptor> = unsafe 
        { 
            core::slice::from_raw_parts_mut(descriptor_buffer as *mut MemoryDescriptor, descriptor_buffer_size as usize / memory_map.descriptor_size as usize) 
        }
            .iter_mut()
            .map(|x| {let mut descriptor: MemoryDescriptor = unsafe { core::mem::zeroed() }; *descriptor = *x; descriptor } )
            .collect();
        descriptors.sort_by_key(|desc| desc.physical_start);
 
        let kernel_start = descriptors
            .iter()
            .find( |desc| { desc.ty == 7 && desc.number_of_pages >= 10 } )
            .map(|desc| desc.physical_start)
            .expect("Failed to find suitable memory region");
        let kernel_pages = 10;
        let kernel_end = kernel_start + kernel_pages * 4096;
        let _ = writeln!(text_out, "Kernel start: 0x{:x}", kernel_start);
        let _ = writeln!(text_out, "Kernel end: 0x{:x}", kernel_end);
        let status = system_table
            .boot_services()
            .allocate_pages(AllocateType::ADDRESS, MemoryType::LOADER_DATA, kernel_pages, kernel_start)
            .expect_success("Failed to allocate kernel memory");
        let _ = writeln!(text_out, "Kernel allocated: {:?}", status);
 
        loop {}
    };
}
