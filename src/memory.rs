use x86_64::{
    structures::paging::{PageTable, OffsetPageTable},
    VirtAddr,
    PhysAddr,
};


/// Initializes a new OffsetPageTable using the provided physical memory offset.
///
/// # Safety
///
/// The caller must ensure that the provided `physical_memory_offset` is valid and correctly maps
/// the physical memory such that the returned OffsetPageTable is valid and unique for the lifetime of the program.
/// Improper use may lead to undefined behavior, including memory corruption.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

/// Returns a mutable reference to the active level 4 page table.
///
/// # Safety
///
/// The caller must ensure that the provided `physical_memory_offset` is valid and correctly maps
/// the physical memory such that the returned reference is valid and unique for the lifetime of the program.
/// Improper use may lead to undefined behavior, including memory corruption.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe{
        &mut *page_table_ptr
    }
}

/// Translates a virtual address to a physical address using the provided physical memory offset.
///
/// # Safety
///
/// The caller must ensure that the provided `physical_memory_offset` is valid and correctly maps
/// the physical memory such that the translation is valid for the lifetime of the program.
/// Improper use may lead to undefined behavior, including memory corruption.
pub unsafe fn transalate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr>{
    transalate_addr_inner(addr, physical_memory_offset)
}

fn transalate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr>{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let table_index = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // Traverses the multi-level page table
    for &index in &table_index{
        // Convert the frame into a page table refrence 
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {
            &*table_ptr
        };

        // Read the page table entry and update 'frame'
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge frames not supported"),

        };
    }
    // Calcaulate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}
