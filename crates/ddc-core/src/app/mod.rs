//! Use cases: implementations of the driving port, generic over the driven
//! port, and the pure logic of the USB switch follow.

mod software_osd;
pub mod usb_follow;
pub mod usb_learn;

pub use software_osd::SoftwareOsd;
