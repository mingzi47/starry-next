use axalloc::GlobalPage;
use axerrno::{AxError, AxResult};
use axhal::{
    mem::{phys_to_virt, virt_to_phys},
    paging::{MappingFlags, PageSize},
    time::{current_ticks, epochoffset_nanos, monotonic_time, wall_time, TimeValue, NANOS_PER_SEC, TIMER_IRQ_NUM},
    trap::{register_trap_handler, IRQ},
};
use axmm::AddrSpace;
use axsync::spin::SpinNoIrq;
use linux_raw_sys::general::{CLOCK_MONOTONIC, CLOCK_REALTIME, CLOCK_TAI};
use memory_addr::{PhysAddr, VirtAddrRange, va};
use spin::Lazy;

use crate::vdso_arch::{VDSO_DATA_SIZE, VDSO_VVAR_OFFSET, vdso_text_size, vdso_text_start};

const VDSO_BASES: usize = CLOCK_TAI as usize + 1;
const DEFAULT_CLOCK_MODE: VdsoClockMode = VdsoClockMode::Pvclock;

#[derive(Debug, Copy, Clone)]
enum VdsoClockMode {
    None = 0,
    Tsc = 1,
    Pvclock = 2,
    Timens = i32::MAX as isize,
}

static VDSO: Lazy<SpinNoIrq<Vdso>> = Lazy::new(|| SpinNoIrq::new(Vdso::default()));

///
pub fn vdso_info() -> &'static SpinNoIrq<Vdso> {
    &VDSO
}

struct Vdso {
    data: &'static mut VdsoData,
    data_start: PhysAddr,
    test_start: PhysAddr,
    frame: GlobalPage,
}

impl Default for Vdso {
    fn default() -> Self {
        let page_size: usize = PageSize::Size4K.into();
        let num_pages = (VDSO_DATA_SIZE + vdso_text_size() + page_size) / page_size;
        let frame = GlobalPage::alloc_contiguous(num_pages, page_size).expect("Alloc Vdso failed!");

        let vdso_data_paddr = frame.start_paddr(virt_to_phys);
        let vdso_text_paddr = vdso_data_paddr + VDSO_DATA_SIZE;

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
        let data_ptr: usize = phys_to_virt(vdso_data_paddr + VDSO_VVAR_OFFSET).into();
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
            data_start: vdso_data_paddr,
            test_start: vdso_text_paddr,
            frame,
        }
    }
}

impl Vdso {
    fn update(&mut self) {
        debug!("update vdso data");
        self.data.seq = 1;
        self.data.last_cycles = current_ticks();

        let shift = self.data.shift;
        self.data.basetime[CLOCK_MONOTONIC as usize].from_time_value(monotonic_time(), shift);
        self.data.basetime[CLOCK_REALTIME as usize].from_time_value(wall_time(), shift);
        debug!("epochoffset_nanos = {}",  epochoffset_nanos());

        self.data.seq = 0;
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct VdsoTimeVal {
    sec: u64,
    nanos_info: u64,
}

impl VdsoTimeVal {
    fn from_time_value(&mut self, tv: TimeValue, shift: u32) {
        self.sec = tv.as_secs();
        self.nanos_info = (tv.subsec_nanos() as u64) << shift;
    }
}

#[repr(C)]
#[derive(Default, Debug)]
struct VdsoData {
    seq: u32,

    clock_mode: i32,
    last_cycles: u64,
    mask: u64,
    mult: u32,
    shift: u32,
    basetime: [VdsoTimeVal; VDSO_BASES],

    tz_minuteswest: i32,
    tz_dsttime: i32,
    hrtimer_res: u32,
    __unused: u32,
}

impl VdsoData {
    fn init(&mut self) {
        self.clock_mode = DEFAULT_CLOCK_MODE as i32;
        self.last_cycles = current_ticks();

        // clac shift mult
        self.clocks_calc_mult_shift(
            axconfig::devices::TIMER_FREQUENCY as u64,
            NANOS_PER_SEC,
            600,
        );
    }

    fn clocks_calc_mult_shift(&mut self, from: u64, to: u64, maxsec: u32) {
        let mut tmp: u64 = (from * maxsec as u64) >> 32;
        let mut sftacc: u32 = 32;
        while tmp > 0 {
            tmp >>= 1;
            sftacc -= 1;
        }

        let mut sft: u32 = 32;
        while sft > 0 {
            tmp = to << sft;
            tmp += from / 2;

            tmp /= from;

            if (tmp >> sftacc) == 0 {
                break;
            }

            sft -= 1;
        }

        self.mult = tmp as u32;
        self.shift = sft;
    }
}


#[register_trap_handler(IRQ)]
fn update_vdso_with_irq(irq_num: usize) -> bool {
    if irq_num == TIMER_IRQ_NUM {
        vdso_info().lock().update();
    }

    true
}

///
pub fn mapping_vdso_uspace(aspace: &mut AddrSpace) -> AxResult<usize> {
    let data_start = VDSO.lock().data_start.as_usize();
    let text_start = VDSO.lock().test_start.as_usize();

    let start_vaddr = aspace
        .find_free_area(
            va!(data_start),
            VDSO_DATA_SIZE + vdso_text_size(),
            VirtAddrRange::new(va!(data_start), aspace.end()),
            PageSize::Size4K,
        )
        .ok_or(AxError::NoMemory)?;

    debug!("mapping vdso : start_vaddr = {:#?}", start_vaddr);

    aspace.map_linear(
        start_vaddr,
        data_start.into(),
        VDSO_DATA_SIZE,
        MappingFlags::READ | MappingFlags::USER,
        PageSize::Size4K,
    )?;

    aspace.map_linear(
        start_vaddr + VDSO_DATA_SIZE,
        text_start.into(),
        vdso_text_size(),
        MappingFlags::READ | MappingFlags::EXECUTE | MappingFlags::USER,
        PageSize::Size4K,
    )?;

    // debug!("vdso_data = ")
    Ok((start_vaddr + VDSO_DATA_SIZE).into())
}
