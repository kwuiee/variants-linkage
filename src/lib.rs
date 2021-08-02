extern crate bam;
extern crate nom;

mod validate;
mod variant;

pub use validate::{ValidateOptions, VariantValidate};
pub use variant::{Edit, Variant};
