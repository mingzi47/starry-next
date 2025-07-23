use core::arch::global_asm;

global_asm!(
    "
	.globl vdso_start, vdso_end
	.section .data
    .balign 4096
vdso_start:
	.incbin \".vscode/vdso/riscv64/vdso.so\"
	.balign 4096
vdso_end:

	.previous
    "
);


///
pub const VDSO_DATA_SIZE: usize = 0x2000;
///
pub const VDSO_VVAR_OFFSET: usize = 0x0;
