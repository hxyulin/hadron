use core::arch::x86_64::__cpuid;

use crate::sync::cell::RacyCell;

// TODO: Replace with a OnceCell or something?
static CPU_INFO: RacyCell<CpuInfo> = RacyCell::new(CpuInfo::default());

/// Initializies the CPU Info using the cpuid
pub unsafe fn init() {
    *CPU_INFO.get_mut() = CpuInfo::get();
}

pub fn cpu_info() -> &'static CpuInfo {
    CPU_INFO.get()
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    features: CpuFeatures,
    extended_feat: ExtendedCpuFeatures,
}

impl const Default for CpuInfo {
    fn default() -> Self {
        Self {
            features: CpuFeatures::empty(),
            extended_feat: ExtendedCpuFeatures::empty(),
        }
    }
}

impl CpuInfo {
    pub fn get() -> Self {
        Self {
            features: CpuFeatures::get(),
            extended_feat: ExtendedCpuFeatures::get(),
        }
    }
}

bitflags::bitflags! {
    /// CPU Features that the CPU supports
    #[derive(Debug, Clone, Copy)]
    pub struct CpuFeatures: u64 {
        /// Supports SSE3 (bit 0 on ECX)
        const SSE3         = 1 << 0;
        /// Supports PCLMUL (bit 1 on ECX)
        const PCLMUL       = 1 << 1;
        /// Supports DTES64 (bit 2 on ECX)
        const DTES64       = 1 << 2;
        const MONITOR      = 1 << 3;
        const DS_CPL       = 1 << 4;
        const VMX          = 1 << 5;
        const SMX          = 1 << 6;
        const EST          = 1 << 7;
        const TM2          = 1 << 8;
        const SSSE3        = 1 << 9;
        const CID          = 1 << 10;
        const SDBG         = 1 << 11;
        const FMA          = 1 << 12;
        const CX16         = 1 << 13;
        const XTPR         = 1 << 14;
        const PDCM         = 1 << 15;
        const PCID         = 1 << 17;
        const DCA          = 1 << 18;
        const SSE4_1       = 1 << 19;
        const SSE4_2       = 1 << 20;
        const X2APIC       = 1 << 21;
        const MOVBE        = 1 << 22;
        const POPCNT       = 1 << 23;
        const TSC_1          = 1 << 24;
        const AES          = 1 << 25;
        const XSAVE        = 1 << 26;
        const OSXSAVE      = 1 << 27;
        const AVX          = 1 << 28;
        const F16C         = 1 << 29;
        const RDRAND       = 1 << 30;
        const HYPERVISOR   = 1 << 31;

        /// The CPU has a FPU (bit 0 on EDX)
        const FPU          = 1 << 32;
        const VME          = 1 << 33;
        const DE           = 1 << 34;
        const PSE          = 1 << 35;
        const TSC_2          = 1 << 36;
        const MSR          = 1 << 37;
        const PAE          = 1 << 38;
        const MCE          = 1 << 39;
        const CX8          = 1 << 40;
        const APIC         = 1 << 41;
        const SEP          = 1 << 43;
        const MTRR         = 1 << 44;
        const PGE          = 1 << 45;
        const MCA          = 1 << 46;
        const CMOV         = 1 << 47;
        const PAT          = 1 << 48;
        const PSE36        = 1 << 49;
        const PSN          = 1 << 50;
        const CLFLUSH      = 1 << 51;
        const DS           = 1 << 53;
        const ACPI         = 1 << 54;
        const MMX          = 1 << 55;
        const FXSR         = 1 << 56;
        const SSE          = 1 << 57;
        const SSE2         = 1 << 58;
        const SS           = 1 << 59;
        const HTT          = 1 << 60;
        const TM           = 1 << 61;
        const IA64         = 1 << 62;
        const PBE          = 1 << 63;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct ExtendedCpuFeatures: u32 {
        const PAGE_1GB = 1 << 26;
    }
}

impl CpuFeatures {
    fn get() -> Self {
        let res = unsafe { __cpuid(1) };
        // FIXME: We cannot trust EDX for some old CPUs
        Self::from_bits_truncate((res.ecx as u64) | ((res.edx as u64) << 32))
    }
}

impl ExtendedCpuFeatures {
    fn get() -> Self {
        let res = unsafe { __cpuid(0x80000001) };
        Self::from_bits_truncate(res.edx)
    }
}
