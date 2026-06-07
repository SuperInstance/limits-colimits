# limits-colimits

**Concrete category theory limits and colimits for the category Set, implemented in Rust.**

This crate provides working, type-safe constructions for the fundamental universal properties of category theory — products, coproducts, pullbacks, pushouts, equalizers, and coequalizers — using concrete types: `Vec<String>` for sets, `HashMap` for morphisms, tuples for products, and tagged unions for coproducts.

No abstract category theory traits. No type-level programming. Just real data structures behaving categorically.

---

## Table of Contents

- [Motivation](#motivation)
- [Theory](#theory)
  - [Universal Properties](#universal-properties)
  - [Cones and Cocones](#cones-and-cocones)
  - [Commutative Diagrams](#commutative-diagrams)
- [Modules](#modules)
- [Design Decisions](#design-decisions)
- [Examples](#examples)
  - [Product of Two Sets](#example-1-product-of-two-sets)
  - [Pullback (Fiber Product)](#example-2-pullback-fiber-product)
  - [Coproduct (Disjoint Union)](#example-3-coproduct-disjoint-union)
- [API Overview](#api-overview)
- [References](#references)
- [License](#license)

---

## Motivation

Category theory is often taught at a level of abstraction that makes it difficult to connect to working code. Textbooks describe products as "objects with projections satisfying a universal property" — but what does that look like in memory?

This crate answers that question. Every construction is concrete:

- A **product** is a `Vec<String>` of tuples like `"(a, 1)"` with projection `HashMap`s.
- A **coproduct** is a tagged `Vec<String>` of elements like `"(A, x)"` and `"(B, y)"`.
- A **pullback** is the subset of a product where two morphisms agree.
- An **equalizer** is the subset of a domain where `f(x) = g(x)`.

If you're learning category theory and want to see universal properties in action, or if you need these constructions for data modeling, this crate is for you.

---

## Theory

### Universal Properties

A **universal property** characterizes an object by its relationship to all other objects of a certain kind, rather than by its internal structure. The idea is: *the object is determined up to unique isomorphism by how it relates to everything else*.

Formally, a universal property says:

> There exists an object $U$ such that for every object $X$ satisfying some condition, there exists a **unique** morphism $X \to U$ (or $U \to X$) making everything commute.

This is the "God object" of category theory — not because it's omniscient, but because everything factors through it uniquely.

**Limits** are universal *cones* (morphisms point inward toward the limit).
**Colimits** are universal *cocones* (morphisms point outward from the colimit).

As Mac Lane writes: *"The idea of universality is one of the central themes of category theory."* [1]

### Cones and Cocones

A **cone** over a diagram $D$ consists of:
- An **apex** object $A$
- A family of **leg** morphisms $\pi_i: A \to D_i$ for each object $D_i$ in the diagram
- Such that for every morphism $f: D_i \to D_j$ in the diagram: $f \circ \pi_i = \pi_j$

```
         A (apex)
        / \
    π₁ /   \ π₂
      /     \
    D₁ --f--> D₂
```

The **commutativity condition** says: going from $A$ to $D₂$ via $D₁$ (path $f \circ \pi_1$) gives the same result as going directly ($\pi_2$).

A **cocone** is the dual — legs point FROM diagram objects TO a nadir:

```
    D₁ --f--> D₂
      \     /
    ι₁ \   / ι₂
        \ /
         B (nadir)
```

Here the condition is: $\iota_2 \circ f = \iota_1$ (both paths from $D_1$ to $B$ agree).

### Commutative Diagrams

A diagram **commutes** when every directed path between the same two vertices composes to the same morphism. Checking commutativity is the central verification step in this crate.

For a cone over a discrete diagram (no morphisms), commutativity is vacuously true — any cone works. The interesting cases come from spans, cospans, and parallel pairs.

---

## Modules

| Module | What It Provides | Key Types |
|--------|-----------------|-----------|
| `diagram` | Diagram shapes and morphisms in Set | `Diagram`, `DiagramShape`, `Morphism` |
| `cone` | Cones and cocones with commutativity checks | `Cone`, `Cocone` |
| `limit` | Universal cones: product, pullback, equalizer | `Limit`, `LimitKind` |
| `colimit` | Universal cocones: coproduct, pushout, coequalizer | `Colimit`, `ColimitKind` |
| `product` | Explicit Cartesian product construction | `Product`, `ProductMorphism` |
| `coproduct` | Explicit disjoint union construction | `Coproduct`, `CoproductMorphism`, `Either` |

### Diagram Shapes

Every diagram has a **shape** that determines which universal construction applies:

```
Discrete:   •₁   •₂   •₃          → Product / Coproduct

Span:       •₁ ← •₀ → •₂          → Pushout (colimit)

Cospan:     •₁ → •₀ ← •₂          → Pullback (limit)

Parallel:   •₁ ⇉ •₂               → Equalizer (limit) / Coequalizer (colimit)
            (f, g)
```

---

## Design Decisions

### Why Concrete Types?

Many category theory libraries in Rust use type-level programming (associated types, generic lifetimes, trait objects) to abstract over categories. This is powerful but opaque.

This crate makes a deliberate choice: **work exclusively in the category Set**, using `Vec<String>` for sets and `HashMap<String, String>` for morphisms. This makes every construction inspectable, printable, and debuggable. You can see exactly what a product, pullback, or coequalizer looks like as data.

### Why String-Based?

Using `String` elements means:
- No generic type parameters cluttering signatures
- Easy to print and inspect results
- Morphisms can be serialized with `serde`
- The focus stays on the categorical structure, not Rust's type system

### Morphisms as HashMaps

A morphism $f: A \to B$ in Set is literally a function — a mapping from elements of $A$ to elements of $B$. Using `HashMap<String, String>` is the most direct representation. Partial morphisms (some elements unmapped) are supported; total morphisms map every domain element.

### Universal Property Verification

Every construction provides a method to verify the universal property:
- `ProductMorphism::verify_universal_property()` checks $\pi_i \circ \langle f_1, \ldots, f_n \rangle = f_i$
- `CoproductMorphism::verify_universal_property()` checks $[f_1, \ldots, f_n] \circ \iota_i = f_i$
- `Limit::factor_morphism()` computes the unique mediating morphism

### Duality

Limits and colimits are **dual** constructions. Every limit concept has a colimit counterpart:

| Limit | Colimit |
|-------|---------|
| Cone (apex, legs inward) | Cocone (nadir, legs outward) |
| Product | Coproduct |
| Pullback | Pushout |
| Equalizer | Coequalizer |
| Subset / Subobject | Quotient / Identification |

The code reflects this duality structurally: `limit.rs` and `colimit.rs` mirror each other, as do `product.rs` and `coproduct.rs`.

---

## Examples

### Example 1: Product of Two Sets

The product of $X = \{a, b\}$ and $Y = \{1, 2\}$ is $X \times Y = \{(a,1), (a,2), (b,1), (b,2)\}$ with projections $\pi_1: X \times Y \to X$ and $\pi_2: X \times Y \to Y$.

```rust
use limits_colimits::product::Product;
use limits_colimits::diagram::Morphism;
use std::collections::HashMap;

// Construct the product
let prod = Product::new(vec![
    ("X", vec!["a".into(), "b".into()]),
    ("Y", vec!["1".into(), "2".into()]),
]);

assert_eq!(prod.size(), 4);
assert!(prod.contains("(a, 1)"));
assert!(prod.contains("(b, 2)"));

// Get projections
let pi_x = prod.projection(0).unwrap();
assert_eq!(pi_x.apply("(a, 2)"), Some(&"a".to_string()));
assert_eq!(pi_x.apply("(b, 1)"), Some(&"b".to_string()));

let pi_y = prod.projection(1).unwrap();
assert_eq!(pi_y.apply("(a, 2)"), Some(&"2".to_string()));

// Use the universal property: given f: Z → X and g: Z → Y,
// the unique morphism ⟨f, g⟩: Z → X×Y satisfies πᵢ∘⟨f,g⟩ = fᵢ
use limits_colimits::product::ProductMorphism;

let f = Morphism::from_fn("f", vec!["*".into()], vec!["a".into(), "b".into()], |_| "a".into());
let g = Morphism::from_fn("g", vec!["*".into()], vec!["1".into(), "2".into()], |_| "2".into());

let pair = ProductMorphism::pair(&prod, vec![f, g]).unwrap();
assert_eq!(pair.apply("*"), Some(&"(a, 2)".to_string()));
assert!(pair.verify_universal_property());
```

### Example 2: Pullback (Fiber Product)

Given morphisms $f: A \to C$ and $g: B \to C$ (a cospan), the pullback is:
$$P = \{(a, b) \in A \times B \mid f(a) = g(b)\}$$

```rust
use limits_colimits::diagram::{Diagram, Morphism};
use limits_colimits::limit::{Limit, LimitKind};
use std::collections::HashMap;

// Define f: A → C mapping a₁→c₁, a₂→c₂
let mut f_map = HashMap::new();
f_map.insert("a1".into(), "c1".into());
f_map.insert("a2".into(), "c2".into());
let f = Morphism::new("f", vec!["a1".into(), "a2".into()],
                       vec!["c1".into(), "c2".into()], f_map);

// Define g: B → C mapping b₁→c₁, b₂→c₂
let mut g_map = HashMap::new();
g_map.insert("b1".into(), "c1".into());
g_map.insert("b2".into(), "c2".into());
let g = Morphism::new("g", vec!["b1".into(), "b2".into()],
                       vec!["c1".into(), "c2".into()], g_map);

// Build the cospan diagram
let diagram = Diagram::cospan("pullback_example",
    vec!["a1".into(), "a2".into()],
    vec!["b1".into(), "b2".into()],
    vec!["c1".into(), "c2".into()],
    f, g);

// Compute the pullback
let limit = Limit::compute(&diagram).unwrap();
assert_eq!(limit.kind, LimitKind::Pullback);
assert_eq!(limit.cone.apex.len(), 2);
assert!(limit.cone.apex.contains(&"(a1, b1)".to_string()));
assert!(limit.cone.apex.contains(&"(a2, b2)".to_string()));

// Verify: f(π_A(a₁,b₁)) = g(π_B(a₁,b₁))
let pi_a = limit.cone.legs.get("A").unwrap();
let pi_b = limit.cone.legs.get("B").unwrap();
assert_eq!(pi_a.apply("(a1, b1)"), Some(&"a1".to_string()));
assert_eq!(pi_b.apply("(a1, b1)"), Some(&"b1".to_string()));
```

### Example 3: Coproduct (Disjoint Union)

The coproduct of $A = \{x\}$ and $B = \{y\}$ is the tagged set $A \sqcup B = \{(A,x), (B,y)\}$ with injection morphisms.

```rust
use limits_colimits::coproduct::{Coproduct, CoproductMorphism, Either};
use limits_colimits::diagram::Morphism;
use std::collections::HashMap;

// Build the coproduct
let coprod = Coproduct::new_binary("A", vec!["x".into()], "B", vec!["y".into()]);
assert_eq!(coprod.size(), 2);
assert_eq!(coprod.elements[0], "(A, x)");
assert_eq!(coprod.elements[1], "(B, y)");

// Injections
let inj_a = coprod.injection_left();
assert_eq!(inj_a.apply("x"), Some(&"(A, x)".to_string()));

let inj_b = coprod.injection_right();
assert_eq!(inj_b.apply("y"), Some(&"(B, y)".to_string()));

// Copairing: given f: A → Z and g: B → Z, define [f,g]: A⊔B → Z
let target = vec!["same".into()];
let f = Morphism::constant("f", vec!["x".into()], target.clone(), "same".into());
let g = Morphism::constant("g", vec!["y".into()], target.clone(), "same".into());

let copair = CoproductMorphism::copair(&coprod, target, vec![f, g]).unwrap();
assert_eq!(copair.apply("(A, x)"), Some(&"same".to_string()));
assert_eq!(copair.apply("(B, y)"), Some(&"same".to_string()));
assert!(copair.verify_universal_property());

// The Either type for tagged union representation
let tagged: Vec<Either<String, String>> = coprod.tagged.clone();
assert_eq!(tagged[0], Either::Left("x".into()));
assert_eq!(tagged[1], Either::Right("y".into()));
```

---

## API Overview

### Diagrams and Morphisms

```rust
// Create a morphism f: A → B
let f = Morphism::new("f", domain_set, codomain_set, mapping);

// Identity morphism
let id = Morphism::identity(set);

// Constant morphism
let c = Morphism::constant("c", domain, codomain, target_element);

// Composition: g ∘ f
let gf = f.compose(&g)?;

// Properties
f.is_total();      // maps every domain element
f.is_injective();  // one-to-one
f.is_surjective(); // onto
f.is_bijective();  // both

// Build diagrams
let d = Diagram::discrete("name", vec![("A", set_a), ("B", set_b)]);
let d = Diagram::cospan("name", a, b, c, f, g);
let d = Diagram::span("name", a, b, c, f, g);
let d = Diagram::parallel_pair("name", a, b, f, g);
```

### Limits

```rust
// Compute any limit (dispatches by shape)
let limit = Limit::compute(&diagram)?;

// Or compute specific kinds
let limit = Limit::compute_product(&discrete_diagram)?;
let limit = Limit::compute_pullback(&cospan_diagram)?;
let limit = Limit::compute_equalizer(&parallel_pair_diagram)?;

// Factor a cone through the limit (universal property)
let factor = limit.factor_morphism(&other_cone)?;
```

### Colimits

```rust
// Compute any colimit
let colimit = Colimit::compute(&diagram)?;

// Specific kinds
let colimit = Colimit::compute_coproduct(&discrete_diagram)?;
let colimit = Colimit::compute_pushout(&span_diagram)?;
let colimit = Colimit::compute_coequalizer(&parallel_pair_diagram)?;

// Factor a cocone through the colimit (universal property)
let factor = colimit.factor_morphism(&other_cocone)?;
```

### Products and Coproducts (Direct)

```rust
// Direct product construction
let prod = Product::new(vec![("A", set_a), ("B", set_b)]);
let pi = prod.projection(0)?;
let morphism = ProductMorphism::pair(&prod, vec![f, g])?;
morphism.verify_universal_property();

// Direct coproduct construction
let coprod = Coproduct::new_binary("A", set_a, "B", set_b);
let inj = coprod.injection_left();
let morphism = CoproductMorphism::copair(&coprod, target, vec![f, g])?;
morphism.verify_universal_property();
```

---

## ASCII Art: Cones and Cocones

### Product (Limit of Discrete Diagram)

```
         Z
        /|\
       / | \
   f₁ /  |  \ fₙ
     /   |   \
    /  ⟨f₁,fₙ⟩ \     ← unique morphism (pairing)
   /     |     \
  X₁     · · ·   Xₙ
  |              |
  π₁             πₙ
  |              |
  +---→ X×Y ←---+
       Product
```

### Coproduct (Colimit of Discrete Diagram)

```
  +---→ X⊔Y ←---+
  |              |
  ι₁             ιₙ
  |              |
  X₁     · · ·   Xₙ
   \     |     /
    \    |    /
  f₁ \   |   / fₙ
      \  |  /
       \ | /
        \|/
         Z
         ↑
    [f₁,...,fₙ]     ← unique morphism (copairing)
```

### Pullback (Limit of Cospan)

```
         P = {(a,b) | f(a) = g(b)}
        / \
   π_A /   \ π_B
      /     \
     A       B
      \     /
    f  \   / g
        \ /
         C
```

The pullback $P$ is the largest subset of $A \times B$ on which $f$ and $g$ agree. In Set, this is the fiber product or "dependent product."

### Pushout (Colimit of Span)

```
         A
        / \
    f  /   \ g
      /     \
     B       C
      \     /
   ι_B \   / ι_C
        \ /
         P = (B ⊔ C) / (f(a) ~ g(a))
```

The pushout glues $B$ and $C$ together along their images from $A$. In topology, this is the "attaching space" construction.

### Equalizer (Limit of Parallel Pair)

```
           E = {a ∈ A | f(a) = g(a)}
           |
           | incl
           |
           A
          / \
      f  /   \ g
        /     \
       +---→ B
```

The equalizer is the largest subset of $A$ on which $f$ and $g$ agree. It is a subobject of $A$.

### Coequalizer (Colimit of Parallel Pair)

```
       A
      / \
  f  /   \ g
    /     \
   +---→ B
          |
          | q (quotient)
          |
          Q = B / (f(a) ~ g(a))
```

The coequalizer identifies elements $f(a)$ and $g(a)$ in $B$ for all $a \in A$. It is a quotient of $B$.

---

## Connections to Other Mathematics

### Set Theory
Products are Cartesian products. Coproducts are disjoint unions. Equalizers are solution sets of equations. Coequalizers are quotients by equivalence relations.

### Topology
Pullbacks are fiber products (restricting bundles). Pushouts are attaching spaces (gluing along a subspace). Products and coproducts in Top are the product topology and disjoint union topology.

### Algebra
In the category of groups, products are direct products and coproducts are free products. Equalizers are subgroups defined by equations. Coequalizers are quotient groups.

### Logic
In categorical logic, products correspond to conjunction (∧), coproducts to disjunction (∨), equalizers to equality predicates, and pullbacks to substitution. The limit/colimit duality mirrors the existential/universal quantifier duality.

---

## References

1. **Mac Lane, S.** (1971). *Categories for the Working Mathematician* (2nd ed., 1998). Springer. — The foundational reference. Chapter III covers universals and limits comprehensively.

2. **Awodey, S.** (2010). *Category Theory* (2nd ed.). Oxford University Press. — An accessible introduction with clear explanations of limits and colimits. Chapters 5 and 6.

3. **Riehl, E.** (2016). *Category Theory in Context*. Dover. — Modern treatment with excellent motivation. Chapter 3 on limits and colimits, with connections to other areas of mathematics.

4. **Leinster, T.** (2014). *Basic Category Theory*. Cambridge University Press. — A concise introduction focusing on the essential concepts. Chapter 5 on limits.

5. **Borceux, F.** (1994). *Handbook of Categorical Algebra*, Volume 1. Cambridge University Press. — Encyclopedic reference for categorical constructions in general categories.

6. **Spivak, D. I.** (2014). *Category Theory for the Sciences*. MIT Press. — Applications-oriented approach that connects categorical abstractions to concrete data structures.

7. **Adámek, J., Herrlich, H., & Strecker, G. E.** (1990). *Abstract and Concrete Categories*. Wiley. — Freely available online. Comprehensive treatment of limits and colimits with many examples in Set.

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](http://opensource.org/licenses/MIT))

at your option.

---

*Educate, don't sell. Every universal property is a contract — and this crate makes those contracts executable.*
