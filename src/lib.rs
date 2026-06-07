//! # limits-colimits
//!
//! Concrete implementations of category theory limits and colimits in the category **Set**.
//!
//! This crate provides working, type-safe constructions for:
//! - **Products** and **Coproducts** (discrete limits/colimits)
//! - **Pullbacks** and **Pushouts** (cospan/span limits/colimits)
//! - **Equalizers** and **Coequalizers** (parallel pair limits/colimits)
//!
//! All types are concrete — `Vec<String>` for sets, `HashMap` for morphisms,
//! tuples for products, and `Either`-tagged vectors for coproducts.
//!
//! ## References
//!
//! - Mac Lane, S. (1971). *Categories for the Working Mathematician*. Springer.
//! - Awodey, S. (2010). *Category Theory* (2nd ed.). Oxford University Press.
//! - Riehl, E. (2016). *Category Theory in Context*. Dover.

pub mod colimit;
pub mod cone;
pub mod coproduct;
pub mod diagram;
pub mod limit;
pub mod product;

pub use colimit::{Colimit, ColimitKind};
pub use cone::{Cocone, Cone};
pub use coproduct::{Coproduct, CoproductMorphism, Either};
pub use diagram::{Diagram, DiagramShape, Morphism};
pub use limit::{Limit, LimitKind};
pub use product::{Product, ProductMorphism};
