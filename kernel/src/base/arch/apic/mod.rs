use local::LocalApic;
use pic::LegacyPic;

pub mod local;
pub mod io;
pub mod pic;

pub struct Apics {
    legacy: Option<LegacyPic>,
    lapic: LocalApic,
}
