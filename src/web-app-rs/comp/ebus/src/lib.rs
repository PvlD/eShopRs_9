#![feature(associated_type_defaults)]
#![feature(box_as_ptr)]

pub mod lib_err;
pub(crate) use lib_err::*;

mod content;
mod dispatcher;

pub use content::*;
pub use dispatcher::*;

#[cfg(test)]
mod events;
#[cfg(test)]
pub use events::*;

#[cfg(test)]
mod test;
#[cfg(test)]
pub(crate) use test::*;
