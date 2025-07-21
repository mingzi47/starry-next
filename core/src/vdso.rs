use core::{alloc::GlobalAlloc, arch::global_asm, slice};

use alloc::boxed::Box;
use axalloc::GlobalPage;
use axerrno::{AxError, AxResult};
use axhal::{
    trap::{IRQ, register_trap_handler}, mem::{phys_to_virt, virt_to_phys}, paging::{MappingFlags, PageSize}, time::{current_ticks, monotonic_time, wall_time, TimeValue, TIMER_IRQ_NUM}
};
use axmm::AddrSpace;
use axsync::spin::SpinNoIrq;
use linux_raw_sys::general::{CLOCK_MONOTONIC, CLOCK_REALTIME};
use memory_addr::{PhysAddr, VirtAddrRange, va};
use spin::Lazy;

// global_asm!(
//     "
// 	.globl vdso_start, vdso_end
// 	.section .data
//     .balign 4096
// vdso_start:
// 	.incbin \"apps/vdso/riscv64/vdso.so\"
// 	.balign 4096
// vdso_end:
//
// 	.previous
//     "
// );

global_asm!(
    "
	.globl vdso_start, vdso_end
	.section .data
    .balign 4096
vdso_start:
	.incbin \".vscode/vdso/x86_64/vdso.so\"
	.balign 4096
vdso_end:

	.previous
    "
);

unsafe extern "C" {
    fn vdso_start();
    fn vdso_end();
}


#[register_trap_handler(IRQ)]
fn update_vdso_with_irq(irq_num: usize) -> bool {
    if irq_num == TIMER_IRQ_NUM {
        vdso_info().lock().update();
    }

    true
}

fn vdso_text_start() -> usize {
    vdso_start as usize
}

fn vdso_text_size() -> usize {
    vdso_end as usize - vdso_start as usize
}

fn vdso_data_size() -> usize {
    0x4000
}

pub fn mapping_vdso_uspace(aspace: &mut AddrSpace) -> AxResult<usize> {
    let data_start = VDSO.lock().vdso_data_paddr.as_usize();
    let text_start = VDSO.lock().vdso_text_paddr.as_usize();

    let start_vaddr = aspace
        .find_free_area(
            va!(data_start),
            vdso_data_size() + vdso_text_size(),
            VirtAddrRange::new(va!(data_start), aspace.end()),
            PageSize::Size4K,
        )
        .ok_or(AxError::NoMemory)?;

    debug!("mapping vdso : start_vaddr = {:#?}", start_vaddr);

    aspace.map_linear(
        start_vaddr,
        data_start.into(),
        vdso_data_size(),
        MappingFlags::READ | MappingFlags::USER,
        PageSize::Size4K,
    )?;

    aspace.map_linear(
        start_vaddr + vdso_data_size(),
        text_start.into(),
        vdso_text_size(),
        MappingFlags::READ | MappingFlags::EXECUTE | MappingFlags::USER,
        PageSize::Size4K,
    )?;

    Ok((start_vaddr + vdso_data_size()).into())
}

static VDSO: Lazy<SpinNoIrq<Vdso>> = Lazy::new(|| SpinNoIrq::new(Vdso::default()));

pub fn vdso_info() -> &'static SpinNoIrq<Vdso> {
    &VDSO
}

struct Vdso {
    data: &'static mut VdsoData,
    vdso_data_paddr: PhysAddr,
    vdso_text_paddr: PhysAddr,
    frame: GlobalPage,
}

impl Default for Vdso {
    fn default() -> Self {
        let page_size: usize = PageSize::Size4K.into();
        let num_pages = (vdso_data_size() + vdso_text_size() + page_size) / page_size;
        let frame = GlobalPage::alloc_contiguous(num_pages, page_size).expect("Alloc Vdso failed!");

        let vdso_data_paddr = frame.start_paddr(virt_to_phys);
        let vdso_text_paddr = vdso_data_paddr + vdso_data_size();

        debug!(
            "vdso = {:#x}, vdso phy = {:#?}",
            vdso_text_start(),
            virt_to_phys(vdso_text_start().into())
        );

        debug!(
            "vdso : [data: {:#?}, text : {:#?}]",
            vdso_data_paddr, vdso_text_paddr
        );

        // init vdso data
        let data_ptr: usize = phys_to_virt(vdso_data_paddr + 0x80).into();
        let data = unsafe { &mut *(data_ptr as *mut VdsoData) };
        data.init();

        // init vdso text
        unsafe {
            core::ptr::copy_nonoverlapping(
                vdso_text_start() as *const u8,
                phys_to_virt(vdso_text_paddr).as_mut_ptr(),
                vdso_text_size(),
            );
        }

        Self {
            data,
            vdso_data_paddr,
            vdso_text_paddr,
            frame,
        }
    }
}

impl Vdso {
    fn update(&mut self) {
        debug!("update vdso data");
        self.data.seq = 1;
        self.data.last_cycles = current_ticks();

        self.data.basetime[CLOCK_MONOTONIC as usize] = monotonic_time();
        self.data.basetime[CLOCK_REALTIME as usize] = wall_time();

        self.data.seq = 0;
        // debug!("update = {:#?}", self.data);
    }
}

const VDSO_BASES: usize = 12;

#[repr(C)]
#[derive(Default, Debug)]
struct VdsoData {
    seq: u32,

    clock_mode: i32,
    last_cycles: u64,
    mask: u64,
    mult: u32,
    shift: u32,
    basetime: [TimeValue; VDSO_BASES],

    tz_minuteswest: i32,
    tz_dsttime: i32,
    hrtimer_res: u32,
    __unused: u32,
}

impl VdsoData {
    fn init(&mut self) {
        self.clock_mode = 1;
    }
}
