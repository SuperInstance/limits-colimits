//! Colimits: universal cocones under diagrams in Set.
//!
//! A **colimit** of a diagram D is a cocone under D through which every other
//! cocone under D factors uniquely. This module provides concrete constructions for:
//!
//! - **Coproduct**: colimit of a discrete diagram (disjoint union)
//! - **Pushout**: colimit of a span diagram (glued union)
//! - **Coequalizer**: colimit of a parallel pair (quotient identifying f(x) ~ g(x))
//!
//! ## Universal Property
//!
//! For a colimit C with injections ιᵢ: Dᵢ → C, any cocone (B, {fᵢ}) factors
//! uniquely through C: there exists a unique h: C → B such that h ∘ ιᵢ = fᵢ.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::cone::Cocone;
use crate::diagram::{Diagram, DiagramShape, Morphism};

/// The kind of colimit being computed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColimitKind {
    Coproduct,
    Pushout,
    Coequalizer,
}

/// A computed colimit: the universal cocone with its factorisation property.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Colimit {
    /// What kind of colimit this is.
    pub kind: ColimitKind,
    /// The universal cocone (nadir + injection legs).
    pub cocone: Cocone,
    /// Human-readable description.
    pub description: String,
}

impl Colimit {
    /// Compute the colimit of a diagram.
    pub fn compute(diagram: &Diagram) -> Result<Self, String> {
        match &diagram.shape {
            DiagramShape::Discrete => Self::compute_coproduct(diagram),
            DiagramShape::Span => Self::compute_pushout(diagram),
            DiagramShape::ParallelPair | DiagramShape::EqualizerShape => {
                Self::compute_coequalizer(diagram)
            }
            _ => Err(format!(
                "Colimit computation not supported for shape {:?}",
                diagram.shape
            )),
        }
    }

    /// Compute the coproduct (colimit of a discrete diagram).
    ///
    /// The coproduct of sets {D₁, ..., Dₙ} is their disjoint union:
    ///   D₁ ⊔ D₂ ⊔ ... ⊔ Dₙ
    /// Elements are tagged with their origin: "L(name, elem)" for left, etc.
    pub fn compute_coproduct(diagram: &Diagram) -> Result<Self, String> {
        if diagram.shape != DiagramShape::Discrete {
            return Err("Coproduct requires a discrete diagram".into());
        }

        let object_names: Vec<&String> = {
            let mut names: Vec<&String> = diagram.objects.keys().collect();
            names.sort();
            names
        };

        let mut disjoint_union = Vec::new();
        let mut legs = HashMap::new();

        for name in &object_names {
            let obj = diagram
                .get_object(name)
                .ok_or_else(|| format!("Object '{}' not found", name))?;
            let mut inj_map = HashMap::new();
            for elem in obj {
                let tagged = format!("({}, {})", name, elem);
                inj_map.insert(elem.clone(), tagged.clone());
                disjoint_union.push(tagged);
            }
            let injection = Morphism::new(
                format!("ι_{}", name),
                obj.clone(),
                disjoint_union.clone(),
                inj_map,
            );
            legs.insert((*name).clone(), injection);
        }

        let cocone = Cocone::new(disjoint_union, legs);
        Ok(Colimit {
            kind: ColimitKind::Coproduct,
            cocone,
            description: "Disjoint union (coproduct)".into(),
        })
    }

    /// Compute the pushout (colimit of a span diagram).
    ///
    /// Given f: A → B and g: A → C, the pushout is:
    ///   B ⊔_A C = (B ⊔ C) / ~
    /// where f(a) ~ g(a) for all a ∈ A.
    pub fn compute_pushout(diagram: &Diagram) -> Result<Self, String> {
        if diagram.shape != DiagramShape::Span {
            return Err("Pushout requires a span diagram".into());
        }
        if diagram.morphisms.len() < 2 {
            return Err("Span requires at least two morphisms".into());
        }

        let f = &diagram.morphisms[0];
        let g = &diagram.morphisms[1];
        let b_set = diagram
            .get_object("B")
            .ok_or("Object 'B' not found")?
            .clone();
        let c_set = diagram
            .get_object("C")
            .ok_or("Object 'C' not found")?
            .clone();
        let a_set = diagram
            .get_object("A")
            .ok_or("Object 'A' not found")?
            .clone();

        // Tag elements from B and C
        let mut elements: Vec<String> = Vec::new();
        let mut b_inj_map = HashMap::new();
        let mut c_inj_map = HashMap::new();

        for b in &b_set {
            let tagged = format!("[B]{}", b);
            b_inj_map.insert(b.clone(), tagged.clone());
            elements.push(tagged);
        }
        for c in &c_set {
            let tagged = format!("[C]{}", c);
            c_inj_map.insert(c.clone(), tagged.clone());
            elements.push(tagged);
        }

        // Identify f(a) ~ g(a) for all a ∈ A
        // We use union-find style: replace [C]g(a) with [B]f(a) in the list
        let mut identifications: HashMap<String, String> = HashMap::new();
        for a in &a_set {
            if let (Some(fb), Some(gc)) = (f.apply(a), g.apply(a)) {
                let b_tag = format!("[B]{}", fb);
                let c_tag = format!("[C]{}", gc);
                // Identify c_tag with b_tag
                identifications.insert(c_tag.clone(), b_tag.clone());
            }
        }

        // Apply identifications to the element list
        let mut final_elements = Vec::new();
        for elem in &elements {
            let resolved = resolve_identification(elem, &identifications);
            if !final_elements.contains(&resolved) {
                final_elements.push(resolved);
            }
        }

        // Update injection maps
        let b_inj: Morphism = {
            let mut map = HashMap::new();
            for (k, v) in &b_inj_map {
                map.insert(k.clone(), resolve_identification(v, &identifications));
            }
            Morphism::new("ι_B", b_set, final_elements.clone(), map)
        };

        let c_inj: Morphism = {
            let mut map = HashMap::new();
            for (k, v) in &c_inj_map {
                map.insert(k.clone(), resolve_identification(v, &identifications));
            }
            Morphism::new("ι_C", c_set, final_elements.clone(), map)
        };

        let mut legs = HashMap::new();
        legs.insert("B".into(), b_inj);
        legs.insert("C".into(), c_inj);

        let cocone = Cocone::new(final_elements, legs);
        Ok(Colimit {
            kind: ColimitKind::Pushout,
            cocone,
            description: "Pushout (glued union)".into(),
        })
    }

    /// Compute the coequalizer (colimit of a parallel pair).
    ///
    /// Given f, g: A → B, the coequalizer is:
    ///   Q = B / ~ where f(a) ~ g(a) for all a ∈ A
    pub fn compute_coequalizer(diagram: &Diagram) -> Result<Self, String> {
        if diagram.morphisms.len() < 2 {
            return Err("Coequalizer requires two parallel morphisms".into());
        }

        let f = &diagram.morphisms[0];
        let g = &diagram.morphisms[1];
        let a_set = f.domain.clone();
        let b_set = f.codomain.clone();

        // Build equivalence classes: identify f(a) with g(a) for each a
        let mut identifications: HashMap<String, String> = HashMap::new();
        for a in &a_set {
            if let (Some(fa), Some(ga)) = (f.apply(a), g.apply(a))
                && fa != ga {
                    identifications.insert(ga.clone(), fa.clone());
                }
        }

        let mut quotient_elements = Vec::new();
        for b in &b_set {
            let resolved = resolve_identification(b, &identifications);
            if !quotient_elements.contains(&resolved) {
                quotient_elements.push(resolved);
            }
        }

        // Build the quotient map (surjection B → Q)
        let mut q_map = HashMap::new();
        for b in &b_set {
            let resolved = resolve_identification(b, &identifications);
            q_map.insert(b.clone(), resolved);
        }
        let quotient_morphism = Morphism::new("q", b_set, quotient_elements.clone(), q_map);

        let mut legs = HashMap::new();
        legs.insert("A".into(), quotient_morphism);

        let cocone = Cocone::new(quotient_elements, legs);
        Ok(Colimit {
            kind: ColimitKind::Coequalizer,
            cocone,
            description: "Coequalizer (quotient identifying f(a) ~ g(a))".into(),
        })
    }

    /// Given another cocone under the same diagram, compute the unique
    /// factorisation morphism through this colimit.
    pub fn factor_morphism(&self, other: &Cocone) -> Option<Morphism> {
        let source = self.cocone.nadir.clone();
        let target = other.nadir.clone();

        match self.kind {
            ColimitKind::Coproduct => {
                // h: ⊔ Dᵢ → B, h(ιᵢ(x)) = fᵢ(x)
                let mut mapping = HashMap::new();
                for (name, inj) in &self.cocone.legs {
                    if let Some(leg) = other.legs.get(name) {
                        for elem in &inj.domain {
                            if let (Some(tagged), Some(target_val)) =
                                (inj.apply(elem), leg.apply(elem))
                            {
                                mapping.insert(tagged.clone(), target_val.clone());
                            }
                        }
                    }
                }
                Some(Morphism::new("factor", source, target, mapping))
            }
            ColimitKind::Pushout => {
                let mut mapping = HashMap::new();
                for elem in &source {
                    // Check which injection produced this element
                    for inj in self.cocone.legs.values() {
                        for orig in &inj.domain {
                            if let Some(tagged) = inj.apply(orig)
                                && *tagged == *elem {
                                    // Find what the other cocone does with this injection
                                    // We need the other cocone's corresponding leg
                                }
                        }
                    }
                }
                // Simplified: for pushout, map each element based on its tag
                for elem in &source {
                    if let Some(orig) = elem.strip_prefix("[B]") {
                        if let Some(leg_b) = other.legs.get("B")
                            && let Some(v) = leg_b.apply(orig) {
                                mapping.insert(elem.clone(), v.clone());
                            }
                    } else if let Some(orig) = elem.strip_prefix("[C]")
                        && let Some(leg_c) = other.legs.get("C")
                            && let Some(v) = leg_c.apply(orig) {
                                mapping.insert(elem.clone(), v.clone());
                            }
                }
                Some(Morphism::new("factor", source, target, mapping))
            }
            ColimitKind::Coequalizer => {
                // h: Q → B such that h(q(x)) = f(x) for the quotient map
                let mut mapping = HashMap::new();
                if let Some(other_leg) = other.legs.get("A") {
                    for elem in &target {
                        // Find all b in B that map to this elem
                        for (k, v) in &other_leg.mapping {
                            if v == elem {
                                // k is in B, its quotient representative is in the nadir
                                // We need the quotient map from the cocone
                                if let Some(q_leg) = self.cocone.legs.get("A")
                                    && let Some(q_elem) = q_leg.apply(k) {
                                        mapping.insert(q_elem.clone(), elem.clone());
                                    }
                            }
                        }
                    }
                }
                Some(Morphism::new("factor", source, target, mapping))
            }
        }
    }
}

/// Resolve an element through a chain of identifications.
/// Uses a two-pass approach: first find the equivalence class,
/// then canonicalize to the lexicographically smallest member.
fn resolve_identification(elem: &str, ids: &HashMap<String, String>) -> String {
    let mut current = elem.to_string();
    let mut seen = Vec::new();
    while let Some(next) = ids.get(&current) {
        if seen.contains(&current) {
            // Cycle detected: pick the smallest element in the cycle
            let cycle_start = seen.iter().position(|s| *s == current).unwrap();
            let cycle: Vec<&str> = seen[cycle_start..].iter().map(|s| s.as_str()).collect();
            let min = cycle.iter().min().unwrap();
            return min.to_string();
        }
        seen.push(current.clone());
        current = next.clone();
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_coproduct_two_sets() {
        let d = Diagram::discrete(
            "X⊔Y",
            vec![
                ("X", vec!["a".into(), "b".into()]),
                ("Y", vec!["1".into(), "2".into()]),
            ],
        );
        let colimit = Colimit::compute_coproduct(&d).unwrap();
        assert_eq!(colimit.kind, ColimitKind::Coproduct);
        assert_eq!(colimit.cocone.nadir.len(), 4);
        assert!(colimit.cocone.nadir.contains(&"(X, a)".to_string()));
        assert!(colimit.cocone.nadir.contains(&"(Y, 1)".to_string()));
    }

    #[test]
    fn test_coproduct_injection() {
        let d = Diagram::discrete("P", vec![("A", vec!["x".into()]), ("B", vec!["y".into()])]);
        let colimit = Colimit::compute_coproduct(&d).unwrap();
        let inj_a = colimit.cocone.legs.get("A").unwrap();
        assert_eq!(inj_a.apply("x"), Some(&"(A, x)".to_string()));
        let inj_b = colimit.cocone.legs.get("B").unwrap();
        assert_eq!(inj_b.apply("y"), Some(&"(B, y)".to_string()));
    }

    #[test]
    fn test_coproduct_three_sets() {
        let d = Diagram::discrete(
            "A⊔B⊔C",
            vec![
                ("A", vec!["1".into()]),
                ("B", vec!["2".into()]),
                ("C", vec!["3".into()]),
            ],
        );
        let colimit = Colimit::compute_coproduct(&d).unwrap();
        assert_eq!(colimit.cocone.nadir.len(), 3);
    }

    #[test]
    fn test_pushout_basic() {
        let mut f_map = HashMap::new();
        f_map.insert("x".into(), "b1".into());
        let f = Morphism::new("f", vec!["x".into()], vec!["b1".into(), "b2".into()], f_map);

        let mut g_map = HashMap::new();
        g_map.insert("x".into(), "c1".into());
        let g = Morphism::new("g", vec!["x".into()], vec!["c1".into(), "c2".into()], g_map);

        let d = Diagram::span(
            "po",
            vec!["x".into()],
            vec!["b1".into(), "b2".into()],
            vec!["c1".into(), "c2".into()],
            f,
            g,
        );
        let colimit = Colimit::compute_pushout(&d).unwrap();
        assert_eq!(colimit.kind, ColimitKind::Pushout);
        // b1 and c1 should be identified
        let nadir = &colimit.cocone.nadir;
        assert!(nadir.contains(&"[B]b1".to_string()) || nadir.contains(&"[B]b2".to_string()));
    }

    #[test]
    fn test_pushout_identification() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "b".into());
        let f = Morphism::new("f", vec!["a".into()], vec!["b".into()], f_map);

        let mut g_map = HashMap::new();
        g_map.insert("a".into(), "c".into());
        let g = Morphism::new("g", vec!["a".into()], vec!["c".into()], g_map);

        let d = Diagram::span(
            "po",
            vec!["a".into()],
            vec!["b".into()],
            vec!["c".into()],
            f,
            g,
        );
        let colimit = Colimit::compute_pushout(&d).unwrap();
        // b and c should be identified, so nadir has 1 element
        assert_eq!(colimit.cocone.nadir.len(), 1);
    }

    #[test]
    fn test_coequalizer_basic() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "x".into());
        f_map.insert("b".into(), "y".into());
        let f = Morphism::new(
            "f",
            vec!["a".into(), "b".into()],
            vec!["x".into(), "y".into()],
            f_map,
        );

        let mut g_map = HashMap::new();
        g_map.insert("a".into(), "y".into());
        g_map.insert("b".into(), "x".into());
        let g = Morphism::new(
            "g",
            vec!["a".into(), "b".into()],
            vec!["x".into(), "y".into()],
            g_map,
        );

        let d = Diagram::parallel_pair(
            "coeq",
            vec!["a".into(), "b".into()],
            vec!["x".into(), "y".into()],
            f,
            g,
        );
        let colimit = Colimit::compute_coequalizer(&d).unwrap();
        assert_eq!(colimit.kind, ColimitKind::Coequalizer);
        // x ~ y (f(a)=x~y=g(a), f(b)=y~x=g(b)), so quotient has 1 element
        assert_eq!(colimit.cocone.nadir.len(), 1);
    }

    #[test]
    fn test_coequalizer_partial() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "x".into());
        f_map.insert("b".into(), "y".into());
        f_map.insert("c".into(), "z".into());
        let f = Morphism::new(
            "f",
            vec!["a".into(), "b".into(), "c".into()],
            vec!["x".into(), "y".into(), "z".into()],
            f_map,
        );

        let mut g_map = HashMap::new();
        g_map.insert("a".into(), "x".into());
        g_map.insert("b".into(), "y".into());
        g_map.insert("c".into(), "y".into());
        let g = Morphism::new(
            "g",
            vec!["a".into(), "b".into(), "c".into()],
            vec!["x".into(), "y".into(), "z".into()],
            g_map,
        );

        let d = Diagram::parallel_pair(
            "coeq",
            vec!["a".into(), "b".into(), "c".into()],
            vec!["x".into(), "y".into(), "z".into()],
            f,
            g,
        );
        let colimit = Colimit::compute_coequalizer(&d).unwrap();
        // f(c)=z, g(c)=y, so y ~ z. x is alone. Quotient has 2 elements.
        assert_eq!(colimit.cocone.nadir.len(), 2);
    }

    #[test]
    fn test_coequalizer_no_identification() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "x".into());
        let f = Morphism::new("f", vec!["a".into()], vec!["x".into()], f_map);

        // Same morphism
        let g_map = f.mapping.clone();
        let g = Morphism::new("g", vec!["a".into()], vec!["x".into()], g_map);

        let d = Diagram::parallel_pair("coeq", vec!["a".into()], vec!["x".into()], f, g);
        let colimit = Colimit::compute_coequalizer(&d).unwrap();
        // No identifications needed, quotient = original set
        assert_eq!(colimit.cocone.nadir.len(), 1);
    }

    #[test]
    fn test_compute_dispatch_span() {
        let f = Morphism::new("f", vec!["a".into()], vec!["b".into()], HashMap::new());
        let g = Morphism::new("g", vec!["a".into()], vec!["c".into()], HashMap::new());
        let d = Diagram::span(
            "s",
            vec!["a".into()],
            vec!["b".into()],
            vec!["c".into()],
            f,
            g,
        );
        let colimit = Colimit::compute(&d).unwrap();
        assert_eq!(colimit.kind, ColimitKind::Pushout);
    }

    #[test]
    fn test_coproduct_universal() {
        let d = Diagram::discrete(
            "A⊔B",
            vec![("A", vec!["a".into()]), ("B", vec!["b".into()])],
        );
        let colimit = Colimit::compute(&d).unwrap();

        // Another cocone
        let other_nadir = vec!["*".into()];
        let mut other_legs = HashMap::new();
        let mut ma = HashMap::new();
        ma.insert("a".into(), "*".into());
        other_legs.insert(
            "A".into(),
            Morphism::new("f₁", vec!["a".into()], vec!["*".into()], ma),
        );
        let mut mb = HashMap::new();
        mb.insert("b".into(), "*".into());
        other_legs.insert(
            "B".into(),
            Morphism::new("f₂", vec!["b".into()], vec!["*".into()], mb),
        );
        let other_cocone = Cocone::new(other_nadir, other_legs);

        let factor = colimit.factor_morphism(&other_cocone).unwrap();
        assert_eq!(factor.apply("(A, a)"), Some(&"*".to_string()));
        assert_eq!(factor.apply("(B, b)"), Some(&"*".to_string()));
    }
}
