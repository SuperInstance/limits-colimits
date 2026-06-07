//! Products: explicit construction for the category Set.
//!
//! The **product** of a family of sets {A₁, ..., Aₙ} is their Cartesian product
//! A₁ × A₂ × ... × Aₙ equipped with projection morphisms πᵢ: ∏ Aⱼ → Aᵢ.
//!
//! ## Universal Property
//!
//! For any object X with morphisms fᵢ: X → Aᵢ, there exists a unique morphism
//! ⟨f₁, ..., fₙ⟩: X → ∏ Aᵢ (the *pairing* of the fᵢ) such that
//! πᵢ ∘ ⟨f₁, ..., fₙ⟩ = fᵢ for all i.
//!
//! ```text
//!       X
//!      /|\
//!   f₁/ | \fₙ          ⟨f₁,...,fₙ⟩ is unique
//!    /  |  \
//!   A₁ · · · Aₙ
//!    \  |  /
//!   π₁\ | /πₙ
//!      \|/
//!     ∏ Aᵢ
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::diagram::Morphism;

/// A concrete product of sets in the category Set.
///
/// Elements are represented as tuples "(a₁, a₂, ..., aₙ)".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    /// The factor sets (name → elements).
    pub factors: Vec<(String, Vec<String>)>,
    /// The product set (Cartesian product of all factors).
    pub elements: Vec<String>,
}

impl Product {
    /// Construct the product of named sets.
    pub fn new(factors: Vec<(impl Into<String>, Vec<String>)>) -> Self {
        let factors: Vec<(String, Vec<String>)> =
            factors.into_iter().map(|(n, e)| (n.into(), e)).collect();

        // Compute Cartesian product
        let mut elements = vec![vec![]];
        for (_, set) in &factors {
            let mut new_elements = Vec::new();
            for prefix in &elements {
                for elem in set {
                    let mut tuple = prefix.clone();
                    tuple.push(elem.clone());
                    new_elements.push(tuple);
                }
            }
            elements = new_elements;
        }

        let element_strs: Vec<String> = elements
            .iter()
            .map(|tuple| format!("({})", tuple.join(", ")))
            .collect();

        Self {
            factors,
            elements: element_strs,
        }
    }

    /// The i-th projection morphism πᵢ: Product → Factorᵢ.
    pub fn projection(&self, factor_index: usize) -> Option<Morphism> {
        if factor_index >= self.factors.len() {
            return None;
        }
        let (name, set) = &self.factors[factor_index];
        let mapping: HashMap<String, String> = self
            .elements
            .iter()
            .map(|elem| {
                let inner = &elem[1..elem.len() - 1];
                let components: Vec<&str> = inner.split(", ").collect();
                (elem.clone(), components[factor_index].to_string())
            })
            .collect();
        Some(Morphism::new(
            format!("π_{}", name),
            self.elements.clone(),
            set.clone(),
            mapping,
        ))
    }

    /// Get all projection morphisms.
    pub fn projections(&self) -> Vec<Morphism> {
        (0..self.factors.len())
            .filter_map(|i| self.projection(i))
            .collect()
    }

    /// The number of elements in the product.
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// The number of factors.
    pub fn num_factors(&self) -> usize {
        self.factors.len()
    }

    /// Check if a given tuple is in the product.
    pub fn contains(&self, element: &str) -> bool {
        self.elements.contains(&element.to_string())
    }
}

/// A morphism into a product, constructed by pairing individual morphisms.
///
/// Given morphisms f₁: X → A₁, ..., fₙ: X → Aₙ, the product morphism
/// ⟨f₁, ..., fₙ⟩: X → A₁ × ... × Aₙ maps x ↦ (f₁(x), ..., fₙ(x)).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductMorphism {
    /// The source set.
    pub source: Vec<String>,
    /// The target product.
    pub target: Product,
    /// The component morphisms.
    pub components: Vec<Morphism>,
    /// The resulting mapping.
    pub mapping: HashMap<String, String>,
}

impl ProductMorphism {
    /// Construct a product morphism from component morphisms.
    ///
    /// All components must have the same domain, and their codomains must
    /// match the factors of the target product (in order).
    pub fn pair(target: &Product, components: Vec<Morphism>) -> Result<Self, String> {
        if components.is_empty() {
            return Err("Need at least one component morphism".into());
        }
        if components.len() != target.factors.len() {
            return Err(format!(
                "Expected {} components, got {}",
                target.factors.len(),
                components.len()
            ));
        }

        let source = components[0].domain.clone();
        for (i, c) in components.iter().enumerate() {
            if c.domain != source {
                return Err(format!(
                    "Component {} has different domain than component 0",
                    i
                ));
            }
            if c.codomain != target.factors[i].1 {
                return Err(format!(
                    "Component {} codomain doesn't match factor {}",
                    i, i
                ));
            }
        }

        let mapping: HashMap<String, String> = source
            .iter()
            .map(|x| {
                let tuple_parts: Vec<String> = components
                    .iter()
                    .filter_map(|c| c.apply(x).cloned())
                    .collect();
                (x.clone(), format!("({})", tuple_parts.join(", ")))
            })
            .collect();

        Ok(Self {
            source,
            target: target.clone(),
            components,
            mapping,
        })
    }

    /// Apply this morphism to an element.
    pub fn apply(&self, element: &str) -> Option<&String> {
        self.mapping.get(element)
    }

    /// Verify the universal property: πᵢ ∘ ⟨f₁, fₙ⟩ = fᵢ.
    pub fn verify_universal_property(&self) -> bool {
        for (i, component) in self.components.iter().enumerate() {
            if let Some(proj) = self.target.projection(i) {
                for x in &self.source {
                    let pair_result = self.apply(x);
                    let direct_result = component.apply(x);
                    if let (Some(p), Some(d)) = (pair_result, direct_result) {
                        let proj_result = proj.apply(p);
                        if proj_result != Some(d) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_new() {
        let p = Product::new(vec![
            ("A", vec!["0".into(), "1".into()]),
            ("B", vec!["x".into(), "y".into()]),
        ]);
        assert_eq!(p.size(), 4);
        assert_eq!(p.num_factors(), 2);
        assert!(p.contains("(0, x)"));
        assert!(p.contains("(1, y)"));
    }

    #[test]
    fn test_product_single_factor() {
        let p = Product::new(vec![("X", vec!["a".into(), "b".into()])]);
        assert_eq!(p.size(), 2);
        assert_eq!(p.elements, vec!["(a)".to_string(), "(b)".to_string()]);
    }

    #[test]
    fn test_product_empty_factor() {
        let p = Product::new(vec![("A", vec!["x".into()]), ("B", Vec::<String>::new())]);
        assert_eq!(p.size(), 0);
    }

    #[test]
    fn test_product_projection() {
        let p = Product::new(vec![
            ("X", vec!["a".into(), "b".into()]),
            ("Y", vec!["1".into(), "2".into()]),
        ]);
        let pi_x = p.projection(0).unwrap();
        assert_eq!(pi_x.apply("(a, 1)"), Some(&"a".to_string()));
        assert_eq!(pi_x.apply("(b, 2)"), Some(&"b".to_string()));
        let pi_y = p.projection(1).unwrap();
        assert_eq!(pi_y.apply("(a, 1)"), Some(&"1".to_string()));
        assert_eq!(pi_y.apply("(b, 2)"), Some(&"2".to_string()));
    }

    #[test]
    fn test_product_projection_out_of_bounds() {
        let p = Product::new(vec![("A", vec!["x".into()])]);
        assert!(p.projection(1).is_none());
    }

    #[test]
    fn test_product_morphism_pair() {
        let target = Product::new(vec![
            ("A", vec!["x".into(), "y".into()]),
            ("B", vec!["0".into(), "1".into()]),
        ]);

        let f = Morphism::from_fn(
            "f",
            vec!["p".into(), "q".into()],
            vec!["x".into(), "y".into()],
            |e| {
                if e == "p" { "x".into() } else { "y".into() }
            },
        );
        let g = Morphism::from_fn(
            "g",
            vec!["p".into(), "q".into()],
            vec!["0".into(), "1".into()],
            |e| {
                if e == "p" { "0".into() } else { "1".into() }
            },
        );

        let pm = ProductMorphism::pair(&target, vec![f, g]).unwrap();
        assert_eq!(pm.apply("p"), Some(&"(x, 0)".to_string()));
        assert_eq!(pm.apply("q"), Some(&"(y, 1)".to_string()));
    }

    #[test]
    fn test_product_morphism_universal() {
        let target = Product::new(vec![("A", vec!["x".into()]), ("B", vec!["0".into()])]);

        let f = Morphism::constant("f", vec!["*".into()], vec!["x".into()], "x".into());
        let g = Morphism::constant("g", vec!["*".into()], vec!["0".into()], "0".into());

        let pm = ProductMorphism::pair(&target, vec![f, g]).unwrap();
        assert!(pm.verify_universal_property());
    }

    #[test]
    fn test_product_morphism_wrong_count() {
        let target = Product::new(vec![("A", vec!["x".into()]), ("B", vec!["y".into()])]);
        let f = Morphism::identity(vec!["*".into()]);
        let result = ProductMorphism::pair(&target, vec![f]);
        assert!(result.is_err());
    }

    #[test]
    fn test_product_all_projections() {
        let p = Product::new(vec![
            ("A", vec!["1".into()]),
            ("B", vec!["2".into()]),
            ("C", vec!["3".into()]),
        ]);
        let projs = p.projections();
        assert_eq!(projs.len(), 3);
    }

    #[test]
    fn test_product_three_factors() {
        let p = Product::new(vec![
            ("A", vec!["0".into(), "1".into()]),
            ("B", vec!["0".into()]),
            ("C", vec!["a".into(), "b".into(), "c".into()]),
        ]);
        assert_eq!(p.size(), 6); // 2 × 1 × 3
        let pi_b = p.projection(1).unwrap();
        assert_eq!(pi_b.apply("(0, 0, a)"), Some(&"0".to_string()));
        let pi_c = p.projection(2).unwrap();
        assert_eq!(pi_c.apply("(1, 0, c)"), Some(&"c".to_string()));
    }
}
