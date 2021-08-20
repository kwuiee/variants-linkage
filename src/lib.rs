extern crate bam;
extern crate nom;

mod link;
mod validate;
pub mod variant;

pub use link::{Link, Linkage};
pub use validate::{ValidateOptions, VariantValidate};
pub use variant::Format as VarFormat;
pub use variant::{Edit, Variant};
