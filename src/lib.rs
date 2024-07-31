#[macro_use]
extern crate log;

mod device;

pub mod battery;
pub mod status;
pub use battery::Battery;
pub use status::ChargingStatus;

mod poll;
