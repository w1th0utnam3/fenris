//! Quadrature rules generated by polyquad.
//!
//! This module contains quadrature rules published in the [paper][paper]
//!
//! ```text
//! Witherden, Freddie D., and Peter E. Vincent.
//! "On the identification of symmetric quadrature rules for finite element methods."
//! Computers & Mathematics with Applications 69, no. 10 (2015): 1232-1241.
//! ```
//!
//! [paper]: https://www.sciencedirect.com/science/article/pii/S0898122115001224#f000035
//! TODO: Explain reference domains etc.

#[derive(Debug)]
pub struct StrengthNotAvailable;

// Load generated code containing quadrature rules generated by build.rs
include!(concat!(env!("OUT_DIR"), "/polyquad/tri.rs"));
include!(concat!(env!("OUT_DIR"), "/polyquad/quad.rs"));
include!(concat!(env!("OUT_DIR"), "/polyquad/tet.rs"));
include!(concat!(env!("OUT_DIR"), "/polyquad/hex.rs"));
include!(concat!(env!("OUT_DIR"), "/polyquad/pri.rs"));
include!(concat!(env!("OUT_DIR"), "/polyquad/pyr.rs"));
