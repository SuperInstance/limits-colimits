//! Limits: universal cones over diagrams in Set.
//!
//! A **limit** of a diagram D is a cone over D through which every other cone
//! over D factors uniquely. This module provides concrete constructions for:
//!
//! - **Product**: limit of a discrete diagram (Cartesian product)
//! - **Pullback**: limit of a cospan diagram (fiber product)
//! - **Equalizer**: limit of a parallel pair (subset where f = g)
//!
//! ## Universal Property
//!
//! For a limit L with projections πᵢ: L → Dᵢ, any cone (A, {fᵢ}) factors
//! uniquely through L: there exists a unique h: A → L such that πᵢ ∘ h = fᵢ.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::cone::Cone;
use crate::diagram::{Diagram, DiagramShape, Morphism};

/// The kind of limit being computed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LimitKind {
    Product,
    Pullback,
    Equalizer,
}

/// A computed limit: the universal cone with its factorisation property.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Limit {
    /// What kind of limit this is.
    pub kind: LimitKind,
    /// The universal cone (apex + projection legs).
    pub cone: Cone,
    /// Human-readable description.
    pub description: String,
}

impl Limit {
    /// Compute the limit of a diagram.
    ///
    /// Dispatches to the appropriate construction based on the diagram shape.
    pub fn compute(diagram: &Diagram) -> Result<Self, String> {
        match &diagram.shape {
            DiagramShape::Discrete => Self::compute_product(diagram),
            DiagramShape::Cospan => Self::compute_pullback(diagram),
            DiagramShape::ParallelPair | DiagramShape::EqualizerShape => {
                Self::compute_equalizer(diagram)
            }
            _ => Err(format!(
                "Limit computation not supported for shape {:?}",
                diagram.shape
            )),
        }
    }

    /// Compute the product (limit of a discrete diagram).
    ///
    /// The product of sets {D₁, ..., Dₙ} is the Cartesian product
    /// D₁ × D₂ × ... × Dₙ with projection morphisms.
    pub fn compute_product(diagram: &Diagram) -> Result<Self, String> {
        if diagram.shape != DiagramShape::Discrete {
            return Err("Product requires a discrete diagram".into());
        }

        let object_names: Vec<&String> = {
            let mut names: Vec<&String> = diagram.objects.keys().collect();
            names.sort();
            names
        };

        if object_names.is_empty() {
            return Err("Cannot compute product of empty diagram".into());
        }

        // Compute Cartesian product
        let mut product_elements: Vec<Vec<String>> = vec![vec![]];
        for name in &object_names {
            let obj = diagram
                .get_object(name)
                .ok_or_else(|| format!("Object '{}' not found", name))?;
            let mut new_elements = Vec::new();
            for prefix in &product_elements {
                for elem in obj {
                    let mut tuple = prefix.clone();
                    tuple.push(elem.clone());
                    new_elements.push(tuple);
                }
            }
            product_elements = new_elements;
        }

        // Represent product elements as "(a, b, c)"
        let product_set: Vec<String> = product_elements
            .iter()
            .map(|tuple| format!("({})", tuple.join(", ")))
            .collect();

        // Build projection morphisms
        let mut legs = HashMap::new();
        for (idx, name) in object_names.iter().enumerate() {
            let obj = diagram.get_object(name).unwrap().clone();
            let mapping: HashMap<String, String> = product_set
                .iter()
                .map(|elem| {
                    // Parse back: "(a, b, c)" → idx-th component
                    let inner = &elem[1..elem.len() - 1];
                    let components: Vec<&str> = inner.split(", ").collect();
                    (elem.clone(), components[idx].to_string())
                })
                .collect();
            let proj = Morphism::new(format!("π_{}", name), product_set.clone(), obj, mapping);
            legs.insert((*name).clone(), proj);
        }

        let cone = Cone::new(product_set, legs);
        Ok(Limit {
            kind: LimitKind::Product,
            cone,
            description: "Cartesian product (limit of discrete diagram)".into(),
        })
    }

    /// Compute the pullback (limit of a cospan diagram).
    ///
    /// Given f: A → C and g: B → C, the pullback is:
    ///   P = { (a, b) ∈ A × B | f(a) = g(b) }
    pub fn compute_pullback(diagram: &Diagram) -> Result<Self, String> {
        if diagram.shape != DiagramShape::Cospan {
            return Err("Pullback requires a cospan diagram".into());
        }
        if diagram.morphisms.len() < 2 {
            return Err("Cospan requires at least two morphisms".into());
        }

        let a_set = diagram
            .get_object("A")
            .ok_or("Object 'A' not found")?
            .clone();
        let b_set = diagram
            .get_object("B")
            .ok_or("Object 'B' not found")?
            .clone();
        let f = &diagram.morphisms[0];
        let g = &diagram.morphisms[1];

        // Pullback = { (a, b) | f(a) = g(b) }
        let mut pullback_elements = Vec::new();
        let mut pa_map = HashMap::new();
        let mut pb_map = HashMap::new();

        for a in &a_set {
            for b in &b_set {
                if let (Some(fa), Some(gb)) = (f.apply(a), g.apply(b))
                    && fa == gb {
                        let elem = format!("({}, {})", a, b);
                        pa_map.insert(elem.clone(), a.clone());
                        pb_map.insert(elem.clone(), b.clone());
                        pullback_elements.push(elem);
                    }
            }
        }

        let proj_a = Morphism::new("π_A", pullback_elements.clone(), a_set, pa_map.clone());
        let proj_b = Morphism::new("π_B", pullback_elements.clone(), b_set, pb_map.clone());

        let mut legs = HashMap::new();
        legs.insert("A".into(), proj_a);
        legs.insert("B".into(), proj_b);
        legs.insert(
            "C".into(),
            // The leg to C is f ∘ π_A (or equivalently g ∘ π_B)
            {
                let mut map_c = HashMap::new();
                for elem in &pullback_elements {
                    if let Some(a_val) = pa_map.get(elem)
                        && let Some(c_val) = f.apply(a_val) {
                            map_c.insert(elem.clone(), c_val.clone());
                        }
                }
                Morphism::new(
                    "π_C",
                    pullback_elements.clone(),
                    diagram.get_object("C").unwrap().clone(),
                    map_c,
                )
            },
        );

        let cone = Cone::new(pullback_elements, legs);
        Ok(Limit {
            kind: LimitKind::Pullback,
            cone,
            description: "Pullback (fiber product)".into(),
        })
    }

    /// Compute the equalizer (limit of a parallel pair).
    ///
    /// Given f, g: A → B, the equalizer is:
    ///   E = { a ∈ A | f(a) = g(a) }
    pub fn compute_equalizer(diagram: &Diagram) -> Result<Self, String> {
        if diagram.morphisms.len() < 2 {
            return Err("Equalizer requires two parallel morphisms".into());
        }

        let f = &diagram.morphisms[0];
        let g = &diagram.morphisms[1];
        let a_set = f.domain.clone();
        let _b_set = f.codomain.clone();

        // Equalizer = { a ∈ A | f(a) = g(a) }
        let eq_elements: Vec<String> = a_set
            .iter()
            .filter(|a| {
                let fa = f.apply(a);
                let ga = g.apply(a);
                fa.is_some() && ga.is_some() && fa == ga
            })
            .cloned()
            .collect();

        let mut legs = HashMap::new();
        // Inclusion morphism from E to A
        let incl_map: HashMap<String, String> =
            eq_elements.iter().map(|e| (e.clone(), e.clone())).collect();
        let incl = Morphism::new("incl", eq_elements.clone(), a_set, incl_map);
        legs.insert("A".into(), incl);

        let cone = Cone::new(eq_elements, legs);
        Ok(Limit {
            kind: LimitKind::Equalizer,
            cone,
            description: "Equalizer (subobject where f = g)".into(),
        })
    }

    /// Given another cone over the same diagram, compute the unique factorisation
    /// morphism through this limit. Returns the morphism from the other cone's
    /// apex to this limit's apex.
    ///
    /// For a product: h(a) = (f₁(a), f₂(a), ..., fₙ(a))
    /// For a pullback: h(x) = (f_A(x), f_B(x)) if f(f_A(x)) = g(f_B(x))
    pub fn factor_morphism(&self, other: &Cone) -> Option<Morphism> {
        let source = other.apex.clone();
        let target = self.cone.apex.clone();

        match self.kind {
            LimitKind::Product => {
                // h(a) = (leg₁(a), leg₂(a), ...)
                let mut mapping = HashMap::new();
                for a in &source {
                    let mut components = Vec::new();
                    let object_names = {
                        let mut names: Vec<&String> = other.legs.keys().collect();
                        names.sort();
                        names
                    };
                    for name in &object_names {
                        if let Some(leg) = other.legs.get(*name)
                            && let Some(v) = leg.apply(a) {
                                components.push(v.clone());
                            }
                    }
                    let target_elem = format!("({})", components.join(", "));
                    if target.contains(&target_elem) {
                        mapping.insert(a.clone(), target_elem);
                    }
                }
                Some(Morphism::new("factor", source, target, mapping))
            }
            LimitKind::Pullback => {
                // h(x) = (leg_A(x), leg_B(x))
                let mut mapping = HashMap::new();
                for x in &source {
                    let a_val = other.legs.get("A").and_then(|l| l.apply(x));
                    let b_val = other.legs.get("B").and_then(|l| l.apply(x));
                    if let (Some(a), Some(b)) = (a_val, b_val) {
                        let target_elem = format!("({}, {})", a, b);
                        if target.contains(&target_elem) {
                            mapping.insert(x.clone(), target_elem);
                        }
                    }
                }
                Some(Morphism::new("factor", source, target, mapping))
            }
            LimitKind::Equalizer => {
                // The factorisation is the restriction of the other cone's leg
                let mut mapping = HashMap::new();
                if let Some(leg_a) = other.legs.get("A") {
                    for a in &source {
                        if let Some(v) = leg_a.apply(a)
                            && target.contains(v) {
                                mapping.insert(a.clone(), v.clone());
                            }
                    }
                }
                Some(Morphism::new("factor", source, target, mapping))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_product_two_sets() {
        let d = Diagram::discrete(
            "X×Y",
            vec![
                ("X", vec!["a".into(), "b".into()]),
                ("Y", vec!["1".into(), "2".into()]),
            ],
        );
        let limit = Limit::compute_product(&d).unwrap();
        assert_eq!(limit.kind, LimitKind::Product);
        assert_eq!(limit.cone.apex.len(), 4);
        assert!(limit.cone.apex.contains(&"(a, 1)".to_string()));
        assert!(limit.cone.apex.contains(&"(b, 2)".to_string()));
    }

    #[test]
    fn test_product_projection() {
        let d = Diagram::discrete(
            "P",
            vec![
                ("A", vec!["x".into(), "y".into()]),
                ("B", vec!["0".into(), "1".into()]),
            ],
        );
        let limit = Limit::compute_product(&d).unwrap();
        let proj_a = limit.cone.legs.get("A").unwrap();
        assert_eq!(proj_a.apply("(x, 0)"), Some(&"x".to_string()));
        assert_eq!(proj_a.apply("(y, 1)"), Some(&"y".to_string()));
        let proj_b = limit.cone.legs.get("B").unwrap();
        assert_eq!(proj_b.apply("(x, 0)"), Some(&"0".to_string()));
        assert_eq!(proj_b.apply("(y, 1)"), Some(&"1".to_string()));
    }

    #[test]
    fn test_product_three_sets() {
        let d = Diagram::discrete(
            "A×B×C",
            vec![
                ("A", vec!["0".into(), "1".into()]),
                ("B", vec!["0".into(), "1".into()]),
                ("C", vec!["0".into(), "1".into()]),
            ],
        );
        let limit = Limit::compute_product(&d).unwrap();
        assert_eq!(limit.cone.apex.len(), 8); // 2³ = 8
    }

    #[test]
    fn test_product_universal_property() {
        let d = Diagram::discrete(
            "X×Y",
            vec![
                ("X", vec!["a".into(), "b".into()]),
                ("Y", vec!["1".into(), "2".into()]),
            ],
        );
        let limit = Limit::compute(&d).unwrap();

        // Another cone: constant to "a" and "1"
        let other_apex = vec!["*".into()];
        let mut other_legs = HashMap::new();
        let mut mx = HashMap::new();
        mx.insert("*".into(), "a".into());
        other_legs.insert(
            "X".into(),
            Morphism::new("f₁", vec!["*".into()], vec!["a".into(), "b".into()], mx),
        );
        let mut my = HashMap::new();
        my.insert("*".into(), "1".into());
        other_legs.insert(
            "Y".into(),
            Morphism::new("f₂", vec!["*".into()], vec!["1".into(), "2".into()], my),
        );
        let other_cone = Cone::new(other_apex, other_legs);

        let factor = limit.factor_morphism(&other_cone).unwrap();
        assert_eq!(factor.apply("*"), Some(&"(a, 1)".to_string()));
    }

    #[test]
    fn test_pullback_basic() {
        let mut f_map = HashMap::new();
        f_map.insert("a1".into(), "c1".into());
        f_map.insert("a2".into(), "c2".into());
        let f = Morphism::new(
            "f",
            vec!["a1".into(), "a2".into()],
            vec!["c1".into(), "c2".into()],
            f_map,
        );

        let mut g_map = HashMap::new();
        g_map.insert("b1".into(), "c1".into());
        g_map.insert("b2".into(), "c2".into());
        let g = Morphism::new(
            "g",
            vec!["b1".into(), "b2".into()],
            vec!["c1".into(), "c2".into()],
            g_map,
        );

        let d = Diagram::cospan(
            "pb",
            vec!["a1".into(), "a2".into()],
            vec!["b1".into(), "b2".into()],
            vec!["c1".into(), "c2".into()],
            f,
            g,
        );
        let limit = Limit::compute_pullback(&d).unwrap();
        assert_eq!(limit.kind, LimitKind::Pullback);
        assert_eq!(limit.cone.apex.len(), 2);
        assert!(limit.cone.apex.contains(&"(a1, b1)".to_string()));
        assert!(limit.cone.apex.contains(&"(a2, b2)".to_string()));
    }

    #[test]
    fn test_pullback_empty_fiber() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "c1".into());
        let f = Morphism::new("f", vec!["a".into()], vec!["c1".into(), "c2".into()], f_map);

        let mut g_map = HashMap::new();
        g_map.insert("b".into(), "c2".into());
        let g = Morphism::new("g", vec!["b".into()], vec!["c1".into(), "c2".into()], g_map);

        let d = Diagram::cospan(
            "pb",
            vec!["a".into()],
            vec!["b".into()],
            vec!["c1".into(), "c2".into()],
            f,
            g,
        );
        let limit = Limit::compute_pullback(&d).unwrap();
        assert!(limit.cone.apex.is_empty());
    }

    #[test]
    fn test_equalizer_basic() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "x".into());
        f_map.insert("b".into(), "y".into());
        f_map.insert("c".into(), "x".into());
        let f = Morphism::new(
            "f",
            vec!["a".into(), "b".into(), "c".into()],
            vec!["x".into(), "y".into()],
            f_map,
        );

        let mut g_map = HashMap::new();
        g_map.insert("a".into(), "x".into());
        g_map.insert("b".into(), "x".into());
        g_map.insert("c".into(), "x".into());
        let g = Morphism::new(
            "g",
            vec!["a".into(), "b".into(), "c".into()],
            vec!["x".into(), "y".into()],
            g_map,
        );

        let d = Diagram::parallel_pair(
            "eq",
            vec!["a".into(), "b".into(), "c".into()],
            vec!["x".into(), "y".into()],
            f,
            g,
        );
        let limit = Limit::compute_equalizer(&d).unwrap();
        assert_eq!(limit.kind, LimitKind::Equalizer);
        let mut expected = vec!["a".to_string(), "c".to_string()];
        let mut actual = limit.cone.apex.clone();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_equalizer_all_equal() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "x".into());
        f_map.insert("b".into(), "y".into());
        let f = Morphism::new(
            "f",
            vec!["a".into(), "b".into()],
            vec!["x".into(), "y".into()],
            f_map,
        );
        let _g = f.clone();
        let g2 = Morphism::new("g", f.domain.clone(), f.codomain.clone(), f.mapping.clone());

        let d = Diagram::parallel_pair(
            "eq",
            vec!["a".into(), "b".into()],
            vec!["x".into(), "y".into()],
            f,
            g2,
        );
        let limit = Limit::compute_equalizer(&d).unwrap();
        assert_eq!(limit.cone.apex.len(), 2);
    }

    #[test]
    fn test_equalizer_none_equal() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "x".into());
        let f = Morphism::new("f", vec!["a".into()], vec!["x".into(), "y".into()], f_map);

        let mut g_map = HashMap::new();
        g_map.insert("a".into(), "y".into());
        let g = Morphism::new("g", vec!["a".into()], vec!["x".into(), "y".into()], g_map);

        let d = Diagram::parallel_pair("eq", vec!["a".into()], vec!["x".into(), "y".into()], f, g);
        let limit = Limit::compute_equalizer(&d).unwrap();
        assert!(limit.cone.apex.is_empty());
    }

    #[test]
    fn test_compute_dispatch_discrete() {
        let d = Diagram::discrete("X", vec![("A", vec!["1".into()])]);
        let limit = Limit::compute(&d).unwrap();
        assert_eq!(limit.kind, LimitKind::Product);
    }
}
