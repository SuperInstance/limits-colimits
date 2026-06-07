//! Coproducts: explicit construction for the category Set.
//!
//! The **coproduct** of a family of sets {A₁, ..., Aₙ} is their disjoint union
//! A₁ ⊔ A₂ ⊔ ... ⊔ Aₙ equipped with injection morphisms ιᵢ: Aᵢ → ⊔ Aⱼ.
//!
//! ## Universal Property
//!
//! For any object Y with morphisms fᵢ: Aᵢ → Y, there exists a unique morphism
//! [f₁, ..., fₙ]: ⊔ Aᵢ → Y (the *copairing*) such that
//! [f₁, ..., fₙ] ∘ ιᵢ = fᵢ for all i.
//!
//! ```text
//!     A₁ · · · Aₙ
//!      \  |  /
//!    ι₁\ | /ιₙ
//!       \|/
//!      ⊔ Aᵢ
//!       /|\
//!     /  |  \
//!   f₁   |   fₙ    [f₁,...,fₙ] is unique
//!        Y
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::diagram::Morphism;

/// Tagged union: an element from either the left or right set.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Either<A, B> {
    /// An element from the left set.
    Left(A),
    /// An element from the right set.
    Right(B),
}

impl<A, B> Either<A, B> {
    /// Check if this is a Left value.
    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    /// Check if this is a Right value.
    pub fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }

    /// Get the left value, if present.
    pub fn left(self) -> Option<A> {
        match self {
            Either::Left(a) => Some(a),
            Either::Right(_) => None,
        }
    }

    /// Get the right value, if present.
    pub fn right(self) -> Option<B> {
        match self {
            Either::Left(_) => None,
            Either::Right(b) => Some(b),
        }
    }

    /// Map over the left variant.
    pub fn map_left<F, C>(self, f: F) -> Either<C, B>
    where
        F: FnOnce(A) -> C,
    {
        match self {
            Either::Left(a) => Either::Left(f(a)),
            Either::Right(b) => Either::Right(b),
        }
    }

    /// Map over the right variant.
    pub fn map_right<F, C>(self, f: F) -> Either<A, C>
    where
        F: FnOnce(B) -> C,
    {
        match self {
            Either::Left(a) => Either::Left(a),
            Either::Right(b) => Either::Right(f(b)),
        }
    }
}

/// A concrete coproduct of sets in the category Set.
///
/// Elements are tagged with their origin set.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Coproduct {
    /// The summand sets (name → elements).
    pub summands: Vec<(String, Vec<String>)>,
    /// The coproduct set (disjoint union of all summands, as tagged strings).
    pub elements: Vec<String>,
    /// The elements as a tagged vector.
    pub tagged: Vec<Either<String, String>>,
}

impl Coproduct {
    /// Construct the coproduct of two named sets.
    pub fn new_binary(
        left_name: impl Into<String>,
        left_set: Vec<String>,
        right_name: impl Into<String>,
        right_set: Vec<String>,
    ) -> Self {
        let ln = left_name.into();
        let rn = right_name.into();

        let mut elements = Vec::new();
        let mut tagged = Vec::new();

        for elem in &left_set {
            elements.push(format!("({}, {})", ln, elem));
            tagged.push(Either::Left(elem.clone()));
        }
        for elem in &right_set {
            elements.push(format!("({}, {})", rn, elem));
            tagged.push(Either::Right(elem.clone()));
        }

        Self {
            summands: vec![(ln, left_set), (rn, right_set)],
            elements,
            tagged,
        }
    }

    /// Construct the coproduct of multiple named sets.
    pub fn new(sets: Vec<(impl Into<String>, Vec<String>)>) -> Self {
        let sets: Vec<(String, Vec<String>)> =
            sets.into_iter().map(|(n, e)| (n.into(), e)).collect();

        let mut elements = Vec::new();
        let mut tagged = Vec::new();
        let mut is_first = true;

        for (name, set) in &sets {
            for elem in set {
                elements.push(format!("({}, {})", name, elem));
                if is_first {
                    tagged.push(Either::Left(elem.clone()));
                } else {
                    tagged.push(Either::Right(elem.clone()));
                }
            }
            is_first = false;
        }

        Self {
            summands: sets,
            elements,
            tagged,
        }
    }

    /// The left injection morphism ι₁: Left → Coproduct.
    pub fn injection_left(&self) -> Morphism {
        let (name, set) = &self.summands[0];
        let mapping: HashMap<String, String> = set
            .iter()
            .map(|e| (e.clone(), format!("({}, {})", name, e)))
            .collect();
        Morphism::new(
            format!("ι_{}", name),
            set.clone(),
            self.elements.clone(),
            mapping,
        )
    }

    /// The right injection morphism ι₂: Right → Coproduct.
    pub fn injection_right(&self) -> Morphism {
        let (name, set) = &self.summands[1];
        let mapping: HashMap<String, String> = set
            .iter()
            .map(|e| (e.clone(), format!("({}, {})", name, e)))
            .collect();
        Morphism::new(
            format!("ι_{}", name),
            set.clone(),
            self.elements.clone(),
            mapping,
        )
    }

    /// Get the injection morphism for the i-th summand.
    pub fn injection(&self, index: usize) -> Option<Morphism> {
        if index >= self.summands.len() {
            return None;
        }
        let (name, set) = &self.summands[index];
        let mapping: HashMap<String, String> = set
            .iter()
            .map(|e| (e.clone(), format!("({}, {})", name, e)))
            .collect();
        Some(Morphism::new(
            format!("ι_{}", name),
            set.clone(),
            self.elements.clone(),
            mapping,
        ))
    }

    /// The number of elements in the coproduct.
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// The number of summands.
    pub fn num_summands(&self) -> usize {
        self.summands.len()
    }
}

/// A morphism from a coproduct, constructed by copairing individual morphisms.
///
/// Given morphisms f₁: A₁ → Y, ..., fₙ: Aₙ → Y, the coproduct morphism
/// [f₁, ..., fₙ]: A₁ ⊔ ... ⊔ Aₙ → Y is defined case-wise:
/// [f₁, ..., fₙ](ιᵢ(a)) = fᵢ(a).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoproductMorphism {
    /// The source coproduct.
    pub source: Coproduct,
    /// The target set.
    pub target: Vec<String>,
    /// The component morphisms (one per summand).
    pub components: Vec<Morphism>,
    /// The resulting mapping.
    pub mapping: HashMap<String, String>,
}

impl CoproductMorphism {
    /// Construct a coproduct morphism by copairing.
    ///
    /// Each component fᵢ: Aᵢ → Y defines how elements from the i-th summand map.
    pub fn copair(
        source: &Coproduct,
        target: Vec<String>,
        components: Vec<Morphism>,
    ) -> Result<Self, String> {
        if components.len() != source.summands.len() {
            return Err(format!(
                "Expected {} components, got {}",
                source.summands.len(),
                components.len()
            ));
        }

        for (i, c) in components.iter().enumerate() {
            if c.domain != source.summands[i].1 {
                return Err(format!(
                    "Component {} domain doesn't match summand {}",
                    i, i
                ));
            }
            if c.codomain != target {
                return Err(format!("Component {} codomain doesn't match target", i));
            }
        }

        let mut mapping = HashMap::new();
        for (i, (_, set)) in source.summands.iter().enumerate() {
            let component = &components[i];
            for elem in set {
                if let Some(result) = component.apply(elem) {
                    let tagged = &source.elements[source
                        .elements
                        .iter()
                        .position(|e| e == &format!("({}, {})", source.summands[i].0, elem))
                        .unwrap()];
                    mapping.insert(tagged.clone(), result.clone());
                }
            }
        }

        Ok(Self {
            source: source.clone(),
            target,
            components,
            mapping,
        })
    }

    /// Apply this morphism to a tagged element.
    pub fn apply(&self, element: &str) -> Option<&String> {
        self.mapping.get(element)
    }

    /// Verify the universal property: [f₁, fₙ] ∘ ιᵢ = fᵢ.
    pub fn verify_universal_property(&self) -> bool {
        for (i, component) in self.components.iter().enumerate() {
            if let Some(inj) = self.source.injection(i) {
                for a in &component.domain {
                    let inj_result = inj.apply(a);
                    if let Some(tagged) = inj_result {
                        let copair_result = self.apply(tagged);
                        let direct_result = component.apply(a);
                        if copair_result != direct_result {
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
    fn test_coproduct_binary() {
        let c = Coproduct::new_binary("A", vec!["x".into()], "B", vec!["y".into()]);
        assert_eq!(c.size(), 2);
        assert_eq!(c.elements, vec!["(A, x)".to_string(), "(B, y)".to_string()]);
    }

    #[test]
    fn test_coproduct_empty_left() {
        let c = Coproduct::new_binary("A", Vec::new(), "B", vec!["y".into()]);
        assert_eq!(c.size(), 1);
    }

    #[test]
    fn test_coproduct_empty_right() {
        let c = Coproduct::new_binary("A", vec!["x".into()], "B", Vec::new());
        assert_eq!(c.size(), 1);
    }

    #[test]
    fn test_coproduct_both_empty() {
        let c = Coproduct::new_binary("A", Vec::new(), "B", Vec::new());
        assert_eq!(c.size(), 0);
    }

    #[test]
    fn test_coproduct_injection_left() {
        let c = Coproduct::new_binary("X", vec!["a".into(), "b".into()], "Y", vec!["1".into()]);
        let inj = c.injection_left();
        assert_eq!(inj.apply("a"), Some(&"(X, a)".to_string()));
        assert_eq!(inj.apply("b"), Some(&"(X, b)".to_string()));
    }

    #[test]
    fn test_coproduct_injection_right() {
        let c = Coproduct::new_binary("X", vec!["a".into()], "Y", vec!["1".into(), "2".into()]);
        let inj = c.injection_right();
        assert_eq!(inj.apply("1"), Some(&"(Y, 1)".to_string()));
        assert_eq!(inj.apply("2"), Some(&"(Y, 2)".to_string()));
    }

    #[test]
    fn test_coproduct_injection_indexed() {
        let c = Coproduct::new_binary("A", vec!["x".into()], "B", vec!["y".into()]);
        let inj_0 = c.injection(0).unwrap();
        assert_eq!(inj_0.apply("x"), Some(&"(A, x)".to_string()));
        let inj_1 = c.injection(1).unwrap();
        assert_eq!(inj_1.apply("y"), Some(&"(B, y)".to_string()));
        assert!(c.injection(2).is_none());
    }

    #[test]
    fn test_coproduct_morphism_copair() {
        let source = Coproduct::new_binary("A", vec!["x".into()], "B", vec!["y".into()]);
        let target = vec!["*".into()];

        let f = Morphism::constant("f", vec!["x".into()], target.clone(), "*".into());
        let g = Morphism::constant("g", vec!["y".into()], target.clone(), "*".into());

        let cm = CoproductMorphism::copair(&source, target, vec![f, g]).unwrap();
        assert_eq!(cm.apply("(A, x)"), Some(&"*".to_string()));
        assert_eq!(cm.apply("(B, y)"), Some(&"*".to_string()));
    }

    #[test]
    fn test_coproduct_morphism_universal() {
        let source = Coproduct::new_binary("A", vec!["x".into()], "B", vec!["y".into()]);
        let target = vec!["p".into(), "q".into()];

        let mut f_map = HashMap::new();
        f_map.insert("x".into(), "p".into());
        let f = Morphism::new("f", vec!["x".into()], target.clone(), f_map);

        let mut g_map = HashMap::new();
        g_map.insert("y".into(), "q".into());
        let g = Morphism::new("g", vec!["y".into()], target.clone(), g_map);

        let cm = CoproductMorphism::copair(&source, target, vec![f, g]).unwrap();
        assert!(cm.verify_universal_property());
    }

    #[test]
    fn test_coproduct_morphism_wrong_count() {
        let source = Coproduct::new_binary("A", vec!["x".into()], "B", vec!["y".into()]);
        let f = Morphism::identity(vec!["x".into()]);
        let result = CoproductMorphism::copair(&source, vec!["*".into()], vec![f]);
        assert!(result.is_err());
    }

    #[test]
    fn test_either_left() {
        let e: Either<String, String> = Either::Left("hello".into());
        assert!(e.is_left());
        assert!(!e.is_right());
        assert_eq!(e.left(), Some("hello".into()));
    }

    #[test]
    fn test_either_right() {
        let e: Either<i32, String> = Either::Right("world".into());
        assert!(e.is_right());
        assert_eq!(e.right(), Some("world".into()));
    }

    #[test]
    fn test_either_map() {
        let e: Either<i32, String> = Either::Left(42);
        let mapped = e.map_left(|x| x * 2);
        assert_eq!(mapped, Either::Left(84));

        let e: Either<i32, String> = Either::Right("hi".into());
        let mapped = e.map_right(|s| s.len() as i32);
        assert_eq!(mapped, Either::Right(2));
    }

    #[test]
    fn test_coproduct_multiple_sets() {
        let c = Coproduct::new(vec![
            ("A", vec!["1".into()]),
            ("B", vec!["2".into()]),
            ("C", vec!["3".into()]),
        ]);
        assert_eq!(c.size(), 3);
        assert_eq!(c.num_summands(), 3);
        assert_eq!(c.elements[0], "(A, 1)");
        assert_eq!(c.elements[1], "(B, 2)");
        assert_eq!(c.elements[2], "(C, 3)");
    }
}
