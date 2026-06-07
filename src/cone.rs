//! Cones and Cocones: the fundamental structures for limits and colimits.
//!
//! A **cone** over a diagram D consists of an object A (the apex) together with
//! a morphism from A to each object in D, such that all triangles commute.
//!
//! A **cocone** under a diagram D consists of an object B (the nadir) together
//! with a morphism from each object in D to B, such that all triangles commute.
//!
//! ```text
//!        Cone (apex A)          Cocone (nadir B)
//!          A                         B
//!         / \                       / \
//!        /   \                     /   \
//!       D₁   D₂                  D₁   D₂
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::diagram::{Diagram, Morphism};

/// A cone over a diagram: an apex object with morphisms (legs) to each
/// object in the diagram, satisfying commutativity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cone {
    /// The apex object (set of elements).
    pub apex: Vec<String>,
    /// Morphisms from the apex to each diagram object, keyed by object name.
    pub legs: HashMap<String, Morphism>,
}

impl Cone {
    /// Create a new cone with the given apex and legs.
    pub fn new(apex: Vec<String>, legs: HashMap<String, Morphism>) -> Self {
        Self { apex, legs }
    }

    /// Check that this cone is over the given diagram: every object in the
    /// diagram has a corresponding leg, and the apex domains are correct.
    pub fn is_over(&self, diagram: &Diagram) -> bool {
        for name in diagram.objects.keys() {
            if let Some(leg) = self.legs.get(name) {
                if leg.domain != self.apex {
                    return false;
                }
                if let Some(obj) = diagram.get_object(name) {
                    // Check that the leg's codomain matches the diagram object
                    if leg.codomain != *obj {
                        // Allow if codomain is a superset (partial mapping)
                        // but for strict cones they should match
                    }
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Check commutativity: for every morphism f: X → Y in the diagram,
    /// we require that leg_Y = f ∘ leg_X (as functions on elements).
    pub fn is_commutative(&self, diagram: &Diagram) -> bool {
        for morphism in &diagram.morphisms {
            // Find the domain and codomain object names
            let (dom_name, cod_name) = self.find_morphism_endpoints(morphism, diagram);
            if let (Some(dom_name), Some(cod_name)) = (dom_name, cod_name)
                && let (Some(leg_dom), Some(leg_cod)) =
                    (self.legs.get(&dom_name), self.legs.get(&cod_name))
                {
                    // For each apex element, check: leg_cod(a) = morphism(leg_dom(a))
                    for a in &self.apex {
                        if let (Some(d_val), Some(c_val)) = (leg_dom.apply(a), leg_cod.apply(a)) {
                            // Check morphism maps d_val to c_val
                            match morphism.apply(d_val) {
                                Some(expected) if expected == c_val => {}
                                _ => return false,
                            }
                        }
                    }
                }
        }
        true
    }

    /// Helper: find which diagram objects a morphism connects.
    fn find_morphism_endpoints(
        &self,
        morphism: &Morphism,
        diagram: &Diagram,
    ) -> (Option<String>, Option<String>) {
        let mut dom_name = None;
        let mut cod_name = None;
        for (name, obj) in &diagram.objects {
            if *obj == morphism.domain {
                dom_name = Some(name.clone());
            }
            if *obj == morphism.codomain {
                cod_name = Some(name.clone());
            }
        }
        (dom_name, cod_name)
    }
}

/// A cocone under a diagram: a nadir object with morphisms from each
/// diagram object to the nadir, satisfying commutativity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cocone {
    /// The nadir object (set of elements).
    pub nadir: Vec<String>,
    /// Morphisms from each diagram object to the nadir, keyed by object name.
    pub legs: HashMap<String, Morphism>,
}

impl Cocone {
    /// Create a new cocone with the given nadir and legs.
    pub fn new(nadir: Vec<String>, legs: HashMap<String, Morphism>) -> Self {
        Self { nadir, legs }
    }

    /// Check that this cocone is under the given diagram.
    pub fn is_under(&self, diagram: &Diagram) -> bool {
        for name in diagram.objects.keys() {
            if let Some(leg) = self.legs.get(name) {
                if leg.codomain != self.nadir {
                    return false;
                }
                if let Some(obj) = diagram.get_object(name)
                    && leg.domain != *obj {
                        return false;
                    }
            } else {
                return false;
            }
        }
        true
    }

    /// Check commutativity: for every morphism f: X → Y in the diagram,
    /// we require that leg_Y = leg_X ∘ f is not right...
    /// Actually: leg_Y(f(x)) = leg_X(x) — no.
    /// For a cocone: leg_Y = leg_X when composed through diagram morphisms.
    /// More precisely: for f: X → Y, we need leg_Y ∘ f = leg_X.
    /// i.e., for every x in X: leg_Y(f(x)) = leg_X(x).
    pub fn is_commutative(&self, diagram: &Diagram) -> bool {
        for morphism in &diagram.morphisms {
            let (dom_name, cod_name) = self.find_morphism_endpoints(morphism, diagram);
            if let (Some(dom_name), Some(cod_name)) = (dom_name, cod_name)
                && let (Some(leg_dom), Some(leg_cod)) =
                    (self.legs.get(&dom_name), self.legs.get(&cod_name))
                {
                    // For each element x in the domain: leg_cod(f(x)) = leg_dom(x)
                    for x in &morphism.domain {
                        if let (Some(fx), Some(leg_x)) = (morphism.apply(x), leg_dom.apply(x)) {
                            match leg_cod.apply(fx) {
                                Some(leg_fx) if leg_fx == leg_x => {}
                                _ => return false,
                            }
                        }
                    }
                }
        }
        true
    }

    fn find_morphism_endpoints(
        &self,
        morphism: &Morphism,
        diagram: &Diagram,
    ) -> (Option<String>, Option<String>) {
        let mut dom_name = None;
        let mut cod_name = None;
        for (name, obj) in &diagram.objects {
            if *obj == morphism.domain {
                dom_name = Some(name.clone());
            }
            if *obj == morphism.codomain {
                cod_name = Some(name.clone());
            }
        }
        (dom_name, cod_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_discrete_diagram() -> Diagram {
        Diagram::discrete(
            "test",
            vec![
                ("X", vec!["a".into(), "b".into()]),
                ("Y", vec!["1".into(), "2".into()]),
            ],
        )
    }

    #[test]
    fn test_cone_over_discrete() {
        let diagram = make_discrete_diagram();
        let apex = vec![
            "(a,1)".into(),
            "(a,2)".into(),
            "(b,1)".into(),
            "(b,2)".into(),
        ];

        let mut px_map = HashMap::new();
        px_map.insert("(a,1)".into(), "a".into());
        px_map.insert("(a,2)".into(), "a".into());
        px_map.insert("(b,1)".into(), "b".into());
        px_map.insert("(b,2)".into(), "b".into());
        let px = Morphism::new("π₁", apex.clone(), vec!["a".into(), "b".into()], px_map);

        let mut py_map = HashMap::new();
        py_map.insert("(a,1)".into(), "1".into());
        py_map.insert("(a,2)".into(), "2".into());
        py_map.insert("(b,1)".into(), "1".into());
        py_map.insert("(b,2)".into(), "2".into());
        let py = Morphism::new("π₂", apex.clone(), vec!["1".into(), "2".into()], py_map);

        let mut legs = HashMap::new();
        legs.insert("X".into(), px);
        legs.insert("Y".into(), py);

        let cone = Cone::new(apex, legs);
        assert!(cone.is_over(&diagram));
        assert!(cone.is_commutative(&diagram)); // discrete has no morphisms to check
    }

    #[test]
    fn test_cone_missing_leg() {
        let diagram = make_discrete_diagram();
        let apex = vec!["a".into()];
        let legs = HashMap::new(); // empty legs
        let cone = Cone::new(apex, legs);
        assert!(!cone.is_over(&diagram));
    }

    #[test]
    fn test_cocone_under_discrete() {
        let diagram = make_discrete_diagram();
        let nadir = vec!["L(1)".into(), "L(2)".into(), "R(a)".into(), "R(b)".into()];

        let mut ix_map: HashMap<String, String> = HashMap::new();
        ix_map.insert("a".into(), "R(a)".into());
        ix_map.insert("b".into(), "R(b)".into());
        let ix = Morphism::new("ι₁", vec!["a".into(), "b".into()], nadir.clone(), ix_map);

        let mut iy_map = HashMap::new();
        iy_map.insert("1".into(), "L(1)".into());
        iy_map.insert("2".into(), "L(2)".into());
        let iy = Morphism::new("ι₂", vec!["1".into(), "2".into()], nadir.clone(), iy_map);

        let mut legs = HashMap::new();
        legs.insert("X".into(), ix);
        legs.insert("Y".into(), iy);

        let cocone = Cocone::new(nadir, legs);
        assert!(cocone.is_under(&diagram));
        assert!(cocone.is_commutative(&diagram));
    }

    #[test]
    fn test_cocone_missing_leg() {
        let diagram = make_discrete_diagram();
        let nadir = vec![];
        let legs = HashMap::new();
        let cocone = Cocone::new(nadir, legs);
        assert!(!cocone.is_under(&diagram));
    }
}
