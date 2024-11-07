mod pos;
mod pos_tags;
mod region;
mod domain;
mod register;
mod language;
mod number;
mod gender;

use serde::{Deserialize, Serialize};
pub use domain::*;
pub use gender::*;
pub use language::*;
pub use number::*;
pub use pos::*;
pub use pos_tags::*;
pub use region::*;
pub use register::*;



#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum PartialWordType {
    Prefix,
    Suffix,
}

