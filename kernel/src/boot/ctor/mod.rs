unsafe extern "C" {
    static __init_array_start: u8;
    static __init_array_end: u8;
}
pub fn init() {
    let start = &raw const __init_array_start as usize;
    let end = &raw const __init_array_end as usize;

    let count = (end - start) / core::mem::size_of::<fn()>();
    log::trace!("CTOR: initializing {} ctors", count);
    let ctors = start as *const fn();

    for i in 0..count {
        unsafe { (*ctors.add(i))() };
    }
}

#[macro_export]
macro_rules! ctor {
    ($name:ident, $fn:expr) => {
        #[used]
        #[unsafe(link_section = ".init_array")]
        static $name: fn() = $fn;
    };
}
