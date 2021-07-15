pub mod automation;
pub mod dyn_color;
pub mod transform_point;
pub use automation::{Anchor, Automation, AutomationSeeker, Weight};
pub use dyn_color::{ColorAnchor, DynColor, DynColorSeeker};
pub use transform_point::{TransPointSeeker, TransformPoint};
