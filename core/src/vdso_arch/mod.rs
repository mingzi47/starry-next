//!
//!
cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
        pub use self::x86_64::*;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        mod riscv;
        pub use self::riscv::*;
    } else if #[cfg(target_arch = "aarch64")]{
        mod aarch64;
        pub use self::aarch64::*;
    } else if #[cfg(target_arch = "loongarch64")] {
        mod loongarch64;
        pub use self::loongarch64::*;
    } else {
        compile_error!("Unsupported architecture");
    }
}

unsafe extern "C" {
    fn vdso_start();
    fn vdso_end();
}

pub fn vdso_text_start() -> usize {
    vdso_start as usize
}

pub fn vdso_text_size() -> usize {
    vdso_end as usize - vdso_start as usize
}
