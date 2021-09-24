//! A composable library for Finite Element computations.
//!
//! **Although featureful, the library API is completely unstable, the functionality is not
//! sufficiently well tested and documentation is sparse. Production usage strongly discouraged
//! at this point.**
use nalgebra::{DimMin, DimName};

pub mod allocators;
pub mod assembly;
pub mod connectivity;
pub mod element;
pub mod error;
pub mod io;
pub mod mesh;
pub mod model;
pub mod quadrature;
pub mod space;
pub mod util;

pub mod workspace;

pub mod geometry {
    pub use fenris_geometry::*;
}

#[cfg(feature = "proptest")]
pub mod proptest;

mod mesh_convert;
mod space_impl;

pub extern crate nalgebra;
pub extern crate nalgebra_sparse;
pub extern crate nested_vec;
pub extern crate vtkio;

/// A small, fixed-size dimension.
///
/// Used as a trait alias for various traits frequently needed by generic `fenris` routines.
pub trait SmallDim: DimName + DimMin<Self, Output = Self> {}

impl<D> SmallDim for D where D: DimName + DimMin<Self, Output = Self> {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Symmetry {
    NonSymmetric,
    Symmetric,
}
