pub trait Console: Send + Sync {
    fn write_str(str: &str);
    fn write_fmt(arguments: &core::fmt::Arguments<'_>);
}
