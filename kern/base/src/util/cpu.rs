use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    fast_syscall: bool,
}

impl CpuFeatures {
    pub fn new() -> Self {
        use core::arch::x86_64::__cpuid;
        let cpuid = unsafe { __cpuid(0x80000001) };
        let fast_syscall = cpuid.edx & (1 << 11) != 0;
        Self { fast_syscall }
    }
}

lazy_static! {
    static ref CPU_FEATURES: CpuFeatures = CpuFeatures::new();
}

pub fn cpu_features() -> &'static CpuFeatures {
    &CPU_FEATURES
}
