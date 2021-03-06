// NOTE: this file is generated by fel4
// NOTE: Don't edit it here; your changes will be lost at the next build!
#![no_std]
#![cfg_attr(feature = "alloc", feature(alloc))]
#![feature(lang_items, core_intrinsics)]
#![feature(global_asm)]
#![feature(panic_info_message)]


extern crate sel4_sys;
#[cfg(feature = "alloc")]
extern crate wee_alloc;
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(feature = "test", feature = "alloc"))]
extern crate proptest;
extern crate hello_world;

use core::alloc::Layout;
use core::intrinsics;
use core::panic::PanicInfo;
use core::mem;
use sel4_sys::*;

#[cfg(feature = "alloc")]
#[global_allocator]
static ALLOCATOR: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// include the seL4 kernel configurations
#[allow(dead_code)]
#[allow(non_upper_case_globals)]
pub mod sel4_config {
    pub const KernelArmSel4Arch:&'static str = "aarch64";
    pub const LibSel4DebugFunctionInstrumentation:&'static str = "none";
    pub const KernelNumPriorities:&'static str = "256";
    pub const KernelRetypeFanOutLimit:&'static str = "256";
    pub const KernelTimeSlice:&'static str = "5";
    pub const KernelVerificationBuild:bool = false;
    pub const KernelDebugDisableBranchPrediction:bool = false;
    pub const ElfloaderImage:&'static str = "binary";
    pub const KernelRootCNodeSizeBits:&'static str = "19";
    pub const KernelOptimisation:&'static str = "-O2";
    pub const KernelFastpath:bool = true;
    pub const KernelDebugBuild:bool = true;
    pub const KernelMaxNumBootinfoUntypedCaps:&'static str = "230";
    pub const KernelUserStackTraceLength:&'static str = "16";
    pub const KernelFPUMaxRestoresSinceSwitch:&'static str = "64";
    pub const KernelFWholeProgram:bool = false;
    pub const KernelResetChunkBits:&'static str = "8";
    pub const LinkPageSize:&'static str = "4096";
    pub const KernelNumDomains:&'static str = "1";
    pub const LibSel4DebugAllocBufferEntries:&'static str = "0";
    pub const KernelMaxNumWorkUnitsPerPreemption:&'static str = "100";
    pub const BuildWithCommonSimulationSettings:bool = true;
    pub const KernelPrinting:bool = true;
    pub const UserLinkerGCSections:bool = false;
    pub const KernelArch:&'static str = "arm";
    pub const LibSel4FunctionAttributes:&'static str = "public";
    pub const KernelColourPrinting:bool = true;
    pub const HardwareDebugAPI:bool = false;
    pub const KernelTimerTickMS:&'static str = "2";
    pub const KernelBenchmarks:&'static str = "none";
    pub const KernelStackBits:&'static str = "12";
    pub const KernelARMPlatform:&'static str = "rpi3";
    pub const KernelMaxNumNodes:&'static str = "1";
}


pub static mut BOOTINFO: *mut seL4_BootInfo = (0 as *mut seL4_BootInfo);
static mut RUN_ONCE: bool = false;

#[no_mangle]
pub unsafe extern "C" fn __sel4_start_init_boot_info(
    bootinfo: *mut seL4_BootInfo,
) {
    if !RUN_ONCE {
        BOOTINFO = bootinfo;
        RUN_ONCE = true;
        seL4_SetUserData((*bootinfo).ipcBuffer as usize as seL4_Word);
    }
}

#[lang = "termination"]
trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}

#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    main();
    panic!("Root task should never return from main!");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(feature = "KernelPrinting")]
    {
        use core::fmt::Write;

        if let Some(loc) = info.location() {
            let _ = write!(
                sel4_sys::DebugOutHandle,
                "panic at {}:{}: ",
                loc.file(),
                loc.line()
            );
        } else {
            let _ = write!(
                sel4_sys::DebugOutHandle,
                "panic: "
            );
        }

        if let Some(fmt) = info.message() {
            let _ = sel4_sys::DebugOutHandle.write_fmt(*fmt);
        }
        let _ = sel4_sys::DebugOutHandle.write_char('\n');

        let _ = write!(
            sel4_sys::DebugOutHandle,
            "----- aborting from panic -----\n"
        );
    }
    unsafe { intrinsics::abort() }
}

#[lang = "eh_personality"]
#[no_mangle]
pub fn eh_personality() {
    #[cfg(feature = "KernelPrinting")]
    {
        use core::fmt::Write;
        let _ = write!(
            sel4_sys::DebugOutHandle,
            "----- aborting from eh_personality -----\n"
        );
    }
    unsafe {
        core::intrinsics::abort();
    }
}

#[lang = "oom"]
#[no_mangle]
pub extern "C" fn oom(_layout: Layout) -> ! {
    #[cfg(feature = "KernelPrinting")]
    {
        use core::fmt::Write;
        let _ = write!(
            sel4_sys::DebugOutHandle,
            "----- aborting from out-of-memory -----\n"
        );
    }
    unsafe {
        core::intrinsics::abort()
    }
}

fn get_untyped(info: &seL4_BootInfo, size_bytes: usize) -> Option<seL4_CPtr> {
    let mut idx = 0;
    for i in info.untyped.start..info.untyped.end {
        if (1 << info.untypedList[idx].sizeBits) >= size_bytes {
            return Some(i);
        }
        idx += 1;
    }
    None
}

const CHILD_STACK_SIZE: usize = 4096;
static mut CHILD_STACK: *const [u64; CHILD_STACK_SIZE] =
    &[0; CHILD_STACK_SIZE];

        
fn main() {
    let bootinfo = unsafe { &*BOOTINFO };
    let cspace_cap = seL4_CapInitThreadCNode;
    let pd_cap = seL4_CapInitThreadVSpace;
    let tcb_cap = bootinfo.empty.start;
    let untyped = get_untyped(bootinfo, 1 << seL4_TCBBits).unwrap();
    let retype_err: seL4_Error = unsafe {
        seL4_Untyped_Retype(
            untyped,
            api_object_seL4_TCBObject.into(),
            seL4_TCBBits.into(),
            cspace_cap.into(),
            cspace_cap.into(),
            seL4_WordBits.into(),
            tcb_cap,
            1,
        )
    };

    assert!(retype_err == 0, "Failed to retype untyped memory");

    let tcb_err: seL4_Error = unsafe {
        seL4_TCB_Configure(
            tcb_cap,
            seL4_CapNull.into(),
            cspace_cap.into(),
            seL4_NilData.into(),
            pd_cap.into(),
            seL4_NilData.into(),
            0,
            0,
        )
    };

    assert!(tcb_err == 0, "Failed to configure TCB");

    let stack_base = unsafe { CHILD_STACK as usize };
    let stack_top = stack_base + CHILD_STACK_SIZE;
    let mut regs: seL4_UserContext = unsafe { mem::zeroed() };
    #[cfg(feature = "test")]
    { regs.pc = hello_world::fel4_test::run as seL4_Word; }
    #[cfg(not(feature = "test"))]
    { regs.pc = hello_world::run as seL4_Word; }
    regs.sp = stack_top as seL4_Word;

    let _: u32 =
        unsafe { seL4_TCB_WriteRegisters(tcb_cap, 0, 0, 2, &mut regs) };
    let _: u32 = unsafe {
        seL4_TCB_SetPriority(tcb_cap, seL4_CapInitThreadTCB.into(), 255)
    };
    let _: u32 = unsafe { seL4_TCB_Resume(tcb_cap) };
    loop {
        unsafe {
            seL4_Yield();
        }
    }
}
        
global_asm!(r###".global _sel4_start
.global _start
.global _stack_bottom
.text

_start:
_sel4_start:
    ldr x19, =_stack_top
    mov sp, x19
    /* x0, the first arg in the calling convention, is set to the bootinfo
     * pointer on startup. */
    bl __sel4_start_init_boot_info
    /* zero argc, argv */
    mov x0, #0
    mov x1, #0
    /* Now go to the "main" stub that rustc generates */
    bl main

.pool
    .data
    .align 16
    .bss
    .align  16

_stack_bottom:
    .space  65536
_stack_top:
"###);

