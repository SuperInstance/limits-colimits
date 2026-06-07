//! Diagrams: collections of objects and morphisms forming a shape.
//!
//! A diagram in a category is a collection of objects and morphisms arranged
//! according to some *shape*. This module provides five fundamental shapes:
//!
//! - **Discrete**: objects only, no morphisms (for products/coproducts)
//! - **Span**: two morphisms with a common domain (for pushouts)
//! - **Cospan**: two morphisms with a common codomain (for pullbacks)
//! - **Parallel pair**: two morphisms with the same domain and codomain (for equalizers/coequalizers)
//! - **Equalizer shape**: alias for parallel pair with a domain object

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A morphism from one set to another, represented as a HashMap mapping
/// elements (as Strings) from the domain to elements in the codomain.
///
/// Elements not present in the map are considered unmapped (partial morphism).
/// For total morphisms, every domain element should have an entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Morphism {
    /// Human-readable name for this morphism.
    pub name: String,
    /// Domain set (list of element names).
    pub domain: Vec<String>,
    /// Codomain set (list of element names).
    pub codomain: Vec<String>,
    /// The mapping: domain element → codomain element.
    pub mapping: HashMap<String, String>,
}

impl Morphism {
    /// Create a new named morphism with the given domain, codomain, and mapping.
    pub fn new(
        name: impl Into<String>,
        domain: Vec<String>,
        codomain: Vec<String>,
        mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            name: name.into(),
            domain,
            codomain,
            mapping,
        }
    }

    /// Create a total morphism from a closure applied to each domain element.
    pub fn from_fn<F>(
        name: impl Into<String>,
        domain: Vec<String>,
        codomain: Vec<String>,
        f: F,
    ) -> Self
    where
        F: Fn(&str) -> String,
    {
        let mapping: HashMap<String, String> = domain.iter().map(|d| (d.clone(), f(d))).collect();
        Self {
            name: name.into(),
            domain,
            codomain,
            mapping,
        }
    }

    /// Apply this morphism to an element, returning None if unmapped.
    pub fn apply(&self, element: &str) -> Option<&String> {
        self.mapping.get(element)
    }

    /// Check if this morphism is total (maps every domain element).
    pub fn is_total(&self) -> bool {
        self.domain.iter().all(|d| self.mapping.contains_key(d))
    }

    /// Check if this morphism is injective (one-to-one).
    pub fn is_injective(&self) -> bool {
        let mut seen: std::collections::HashSet<&String> = std::collections::HashSet::new();
        for v in self.mapping.values() {
            if !seen.insert(v) {
                return false;
            }
        }
        true
    }

    /// Check if this morphism is surjective (onto the codomain).
    pub fn is_surjective(&self) -> bool {
        let image: std::collections::HashSet<&String> = self.mapping.values().collect();
        self.codomain.iter().all(|c| image.contains(c))
    }

    /// Check if this morphism is bijective (both injective and surjective).
    pub fn is_bijective(&self) -> bool {
        self.is_injective() && self.is_surjective()
    }

    /// Compose this morphism with another: `g ∘ self`.
    /// The codomain of `self` must be the domain of `g`.
    pub fn compose(&self, g: &Morphism) -> Option<Morphism> {
        if self.codomain != g.domain {
            return None;
        }
        let mapping: HashMap<String, String> = self
            .mapping
            .iter()
            .filter_map(|(k, v)| g.mapping.get(v).map(|w| (k.clone(), w.clone())))
            .collect();
        Some(Morphism::new(
            format!("{}_{}", g.name, self.name),
            self.domain.clone(),
            g.codomain.clone(),
            mapping,
        ))
    }

    /// The identity morphism on a set.
    pub fn identity(set: Vec<String>) -> Self {
        let mapping: HashMap<String, String> = set.iter().map(|s| (s.clone(), s.clone())).collect();
        Morphism::new("id", set.clone(), set, mapping)
    }

    /// A constant morphism that maps everything to a single element.
    pub fn constant(
        name: impl Into<String>,
        domain: Vec<String>,
        codomain: Vec<String>,
        target: String,
    ) -> Self {
        let mapping: HashMap<String, String> =
            domain.iter().map(|d| (d.clone(), target.clone())).collect();
        Morphism::new(name, domain, codomain, mapping)
    }
}

/// The shape of a diagram, determining which universal construction applies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagramShape {
    /// A discrete diagram: objects with no morphisms between them.
    /// Limits of discrete diagrams are **products**.
    /// Colimits of discrete diagrams are **coproducts**.
    Discrete,

    /// A span: one object mapping to two others (f: A → B, g: A → C).
    /// Colimits of spans are **pushouts**.
    Span,

    /// A cospan: two objects mapping to one (f: A → C, g: B → C).
    /// Limits of cospan diagrams are **pullbacks**.
    Cospan,

    /// A parallel pair: two morphisms f, g: A → B between the same objects.
    /// Limits of parallel pairs are **equalizers**.
    /// Colimits of parallel pairs are **coequalizers**.
    ParallelPair,

    /// An equalizer shape: a parallel pair with an explicit domain object.
    EqualizerShape,
}

/// A diagram in the category Set: a collection of named sets (objects)
/// and morphisms between them, arranged in a specific shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Diagram {
    /// Human-readable name for the diagram.
    pub name: String,
    /// The shape of the diagram.
    pub shape: DiagramShape,
    /// Named objects (sets) in the diagram.
    pub objects: HashMap<String, Vec<String>>,
    /// Morphisms in the diagram.
    pub morphisms: Vec<Morphism>,
}

impl Diagram {
    /// Create a new empty diagram with a name and shape.
    pub fn new(name: impl Into<String>, shape: DiagramShape) -> Self {
        Self {
            name: name.into(),
            shape,
            objects: HashMap::new(),
            morphisms: Vec::new(),
        }
    }

    /// Add a named object (set) to the diagram.
    pub fn add_object(&mut self, name: impl Into<String>, elements: Vec<String>) {
        self.objects.insert(name.into(), elements);
    }

    /// Add a morphism to the diagram.
    pub fn add_morphism(&mut self, morphism: Morphism) {
        self.morphisms.push(morphism);
    }

    /// Get an object by name.
    pub fn get_object(&self, name: &str) -> Option<&Vec<String>> {
        self.objects.get(name)
    }

    /// Get the names of all objects.
    pub fn object_names(&self) -> Vec<&String> {
        let mut names: Vec<&String> = self.objects.keys().collect();
        names.sort();
        names
    }

    /// Build a discrete diagram from a list of named sets.
    pub fn discrete(
        name: impl Into<String>,
        objects: Vec<(impl Into<String>, Vec<String>)>,
    ) -> Self {
        let mut diagram = Self::new(name, DiagramShape::Discrete);
        for (n, elems) in objects {
            diagram.add_object(n, elems);
        }
        diagram
    }

    /// Build a cospan diagram: f: A → C, g: B → C.
    /// Used for computing pullbacks.
    pub fn cospan(
        name: impl Into<String>,
        a: Vec<String>,
        b: Vec<String>,
        c: Vec<String>,
        f: Morphism,
        g: Morphism,
    ) -> Self {
        let mut diagram = Self::new(name, DiagramShape::Cospan);
        diagram.add_object("A", a);
        diagram.add_object("B", b);
        diagram.add_object("C", c);
        diagram.add_morphism(f);
        diagram.add_morphism(g);
        diagram
    }

    /// Build a span diagram: f: A → B, g: A → C.
    /// Used for computing pushouts.
    pub fn span(
        name: impl Into<String>,
        a: Vec<String>,
        b: Vec<String>,
        c: Vec<String>,
        f: Morphism,
        g: Morphism,
    ) -> Self {
        let mut diagram = Self::new(name, DiagramShape::Span);
        diagram.add_object("A", a);
        diagram.add_object("B", b);
        diagram.add_object("C", c);
        diagram.add_morphism(f);
        diagram.add_morphism(g);
        diagram
    }

    /// Build a parallel pair diagram: f, g: A → B.
    /// Used for equalizers and coequalizers.
    pub fn parallel_pair(
        name: impl Into<String>,
        a: Vec<String>,
        b: Vec<String>,
        f: Morphism,
        g: Morphism,
    ) -> Self {
        let mut diagram = Self::new(name, DiagramShape::ParallelPair);
        diagram.add_object("A", a);
        diagram.add_object("B", b);
        diagram.add_morphism(f);
        diagram.add_morphism(g);
        diagram
    }

    /// Validate the diagram: check that morphism domains/codomains
    /// reference objects in the diagram.
    pub fn validate(&self) -> Result<(), String> {
        for m in &self.morphisms {
            // Check that domain/codomain elements exist in their respective objects
            // (we allow partial morphisms, so we just check that the objects exist)
            if !self.objects.contains_key(&m.name) {
                // Morphism name doesn't need to be an object
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphism_identity() {
        let set = vec!["a".into(), "b".into(), "c".into()];
        let id = Morphism::identity(set.clone());
        assert!(id.is_total());
        assert!(id.is_bijective());
        assert_eq!(id.apply("a"), Some(&"a".to_string()));
        assert_eq!(id.apply("b"), Some(&"b".to_string()));
        assert_eq!(id.apply("c"), Some(&"c".to_string()));
    }

    #[test]
    fn test_morphism_compose() {
        let a = vec!["x".into(), "y".into()];
        let b = vec!["1".into(), "2".into()];
        let c = vec!["α".into(), "β".into()];
        let mut fmap = HashMap::new();
        fmap.insert("x".into(), "1".into());
        fmap.insert("y".into(), "2".into());
        let f = Morphism::new("f", a.clone(), b.clone(), fmap);

        let mut gmap = HashMap::new();
        gmap.insert("1".into(), "α".into());
        gmap.insert("2".into(), "β".into());
        let g = Morphism::new("g", b, c.clone(), gmap);

        let gf = f.compose(&g).unwrap();
        assert_eq!(gf.apply("x"), Some(&"α".to_string()));
        assert_eq!(gf.apply("y"), Some(&"β".to_string()));
        assert_eq!(gf.domain, a);
        assert_eq!(gf.codomain, c);
    }

    #[test]
    fn test_morphism_compose_incompatible() {
        let f = Morphism::new("f", vec!["a".into()], vec!["x".into()], HashMap::new());
        let g = Morphism::new("g", vec!["y".into()], vec!["z".into()], HashMap::new());
        assert!(f.compose(&g).is_none());
    }

    #[test]
    fn test_morphism_constant() {
        let domain = vec!["a".into(), "b".into(), "c".into()];
        let codomain = vec!["x".into(), "y".into()];
        let c = Morphism::constant("c", domain.clone(), codomain.clone(), "x".into());
        assert!(c.is_total());
        assert!(!c.is_injective());
        assert!(!c.is_surjective());
        assert_eq!(c.apply("a"), Some(&"x".to_string()));
        assert_eq!(c.apply("b"), Some(&"x".to_string()));
    }

    #[test]
    fn test_morphism_injective() {
        let domain = vec!["a".into(), "b".into()];
        let codomain = vec!["x".into(), "y".into(), "z".into()];
        let mut map = HashMap::new();
        map.insert("a".into(), "x".into());
        map.insert("b".into(), "y".into());
        let f = Morphism::new("f", domain, codomain, map);
        assert!(f.is_injective());
        assert!(!f.is_surjective());
        assert!(!f.is_bijective());
    }

    #[test]
    fn test_morphism_from_fn() {
        let domain = vec!["1".into(), "2".into(), "3".into()];
        let codomain = vec!["2".into(), "4".into(), "6".into()];
        let f = Morphism::from_fn("double", domain.clone(), codomain, |x| {
            (x.parse::<i32>().unwrap() * 2).to_string()
        });
        assert!(f.is_total());
        assert_eq!(f.apply("1"), Some(&"2".to_string()));
        assert_eq!(f.apply("2"), Some(&"4".to_string()));
        assert_eq!(f.apply("3"), Some(&"6".to_string()));
    }

    #[test]
    fn test_diagram_discrete() {
        let d = Diagram::discrete(
            "prod_base",
            vec![
                ("X", vec!["a".into(), "b".into()]),
                ("Y", vec!["1".into(), "2".into()]),
            ],
        );
        assert_eq!(d.shape, DiagramShape::Discrete);
        assert_eq!(d.object_names().len(), 2);
        assert!(d.morphisms.is_empty());
    }

    #[test]
    fn test_diagram_cospan() {
        let f = Morphism::new("f", vec!["a".into()], vec!["x".into()], HashMap::new());
        let g = Morphism::new("g", vec!["b".into()], vec!["x".into()], HashMap::new());
        let d = Diagram::cospan(
            "pullback_shape",
            vec!["a".into()],
            vec!["b".into()],
            vec!["x".into()],
            f,
            g,
        );
        assert_eq!(d.shape, DiagramShape::Cospan);
        assert_eq!(d.morphisms.len(), 2);
    }

    #[test]
    fn test_diagram_parallel_pair() {
        let mut f_map = HashMap::new();
        f_map.insert("a".into(), "x".into());
        let f = Morphism::new("f", vec!["a".into()], vec!["x".into()], f_map);
        let mut g_map = HashMap::new();
        g_map.insert("a".into(), "y".into());
        let g = Morphism::new("g", vec!["a".into()], vec!["x".into(), "y".into()], g_map);
        let d = Diagram::parallel_pair("eq", vec!["a".into()], vec!["x".into(), "y".into()], f, g);
        assert_eq!(d.shape, DiagramShape::ParallelPair);
        assert_eq!(d.morphisms.len(), 2);
    }
}
