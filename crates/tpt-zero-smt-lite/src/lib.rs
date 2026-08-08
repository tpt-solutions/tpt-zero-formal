//! Boolean and pseudo-boolean (0/1) linear constraint helpers with a tiny
//! `no_std` SMT-lite solver for checking satisfiability at runtime. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! **Note on the "linear" constraints:** they are *pseudo-boolean* constraints,
//! not general integer-linear constraints. Each variable is a boolean and is
//! read as the integer `1`/`0`, so `pb_eq`/`pb_leq` describe linear relations over
//! the `0`/`1` assignment — not over arbitrary `i64` values.
//!
//! The crate models two kinds of constraint over a fixed set of
//! [`Var`]iables:
//!
//! * **Boolean constraints** over the truth value of each [`Var`], built with
//!   the combinators on [`ConstraintSet`]: [`ConstraintSet::and`],
//!   [`ConstraintSet::or`], [`ConstraintSet::implies`], [`ConstraintSet::not`],
//!   [`ConstraintSet::all`], [`ConstraintSet::any`], [`ConstraintSet::var`].
//! * **Pseudo-boolean (0/1) linear constraints** over the boolean assignment
//!   to each [`Var`] (read as `1`/`0`), built with [`ConstraintSet::pb_eq`] and
//!   [`ConstraintSet::pb_leq`].
//!
//! A [`ConstraintSet`] is also the arena that owns the boolean expression
//! tree, so no heap allocation is required and the crate builds with
//! `--no-default-features` (pure `core`).
//!
//! [`solve`] (or [`ConstraintSet::solve`]) performs a naive enumeration over
//! the `2^n` boolean assignments, returning a [`Witness`] when the set is
//! satisfiable.
//!
//! ```
//! use tpt_zero_smt_lite::{Var, ConstraintSet};
//!
//! let a = Var(0);
//! let b = Var(1);
//!
//! let cs = ConstraintSet::new(2);
//! let expr = cs.and(cs.or(cs.var(a), cs.var(b)), cs.not(cs.var(a)));
//! cs.add_bool(expr);
//! cs.pb_leq(&[(1, a), (1, b)], 1);
//!
//! let witness = cs.solve().expect("the constraints are satisfiable");
//! assert_eq!(witness.bool_of(a), Some(false));
//! assert_eq!(witness.bool_of(b), Some(true));
//! ```

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// A variable identifier.
///
/// Variables are integers in `0..N`. The value `N` (the number of variables)
/// is supplied when building a [`ConstraintSet`] so the solver knows the size
/// of the search space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Var(pub usize);

/// An opaque handle to a boolean expression node stored in a
/// [`ConstraintSet`]'s arena.
///
/// Obtain handles via the combinators on [`ConstraintSet`] (for example
/// [`ConstraintSet::var`], [`ConstraintSet::and`]) rather than constructing
/// them directly; a handle is only meaningful within the set that created it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Expr {
    node: u16,
}

/// A node in the boolean-expression arena owned by a [`ConstraintSet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Node {
    True,
    False,
    Var(Var),
    Not(u16),
    And(u16, u16),
    Or(u16, u16),
    Implies(u16, u16),
}

/// The left-hand side of a linear integer constraint: a sum of terms
/// `coeff * Var`.
///
/// A term is a `(coefficient, variable)` pair. The list is allowed to contain
/// duplicate variables; their coefficients are summed during evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearTerm {
    /// The integer coefficient multiplying the variable.
    pub coeff: i64,
    /// The variable being multiplied.
    pub var: Var,
}

impl LinearTerm {
    /// Creates a new linear term `coeff * var`.
    #[must_use]
    pub const fn new(coeff: i64, var: Var) -> Self {
        Self { coeff, var }
    }
}

/// A fixed-size list of linear terms, stored inline to stay heap-free.
///
/// The capacity is generous enough for realistic small problems. Duplicate
/// variables in the list are summed during evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearTerms {
    inner: [Option<LinearTerm>; LinearTerms::CAP],
    len: usize,
}

impl LinearTerms {
    const CAP: usize = 16;

    /// Creates an empty list of linear terms.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: [None; Self::CAP],
            len: 0,
        }
    }

    /// The number of terms stored.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if no terms are stored.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends a term, returning `false` (and leaving the list unchanged) if
    /// the capacity is exhausted.
    pub fn push(&mut self, term: LinearTerm) -> bool {
        if self.len >= Self::CAP {
            return false;
        }
        self.inner[self.len] = Some(term);
        self.len += 1;
        true
    }

    /// Iterates over the stored terms.
    pub fn iter(&self) -> impl Iterator<Item = LinearTerm> + '_ {
        self.inner
            .iter()
            .take(self.len)
            .filter_map(Option::as_ref)
            .copied()
    }
}

impl Default for LinearTerms {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&[(i64, Var)]> for LinearTerms {
    fn from(pairs: &[(i64, Var)]) -> Self {
        let mut terms = LinearTerms::new();
        for &(coeff, var) in pairs {
            let _ = terms.push(LinearTerm::new(coeff, var));
        }
        terms
    }
}

    /// A single constraint: either a boolean expression that must hold, or a
    /// pseudo-boolean (0/1) linear relation that must hold over the boolean
    /// assignment (each variable read as `1`/`0`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Constraint {
        /// A boolean expression that must evaluate to `true`.
        Bool(Expr),
        /// A pseudo-boolean equality `sum(coeff * var) == rhs` that must hold,
        /// where each `var` is read as `1` (true) or `0` (false).
        PbEq {
            /// The sum of linear terms.
            terms: LinearTerms,
            /// The required right-hand side value.
            rhs: i64,
        },
        /// A pseudo-boolean inequality `sum(coeff * var) <= rhs` that must hold,
        /// where each `var` is read as `1` (true) or `0` (false).
        PbLeq {
            /// The sum of linear terms.
            terms: LinearTerms,
            /// The upper bound for the right-hand side value.
            rhs: i64,
        },
    }

/// A collection of [`Constraint`]s together with an arena of boolean
/// expression nodes and the number of variables they range over.
///
/// Construct one with [`ConstraintSet::new`], build expressions with the
/// `ConstraintSet` combinators, and add constraints with [`add_bool`],
/// [`pb_eq`], or [`pb_leq`]. Call [`solve`] to check satisfiability.
///
/// [`add_bool`]: ConstraintSet::add_bool
/// [`pb_eq`]: ConstraintSet::pb_eq
/// [`pb_leq`]: ConstraintSet::pb_leq
/// [`solve`]: ConstraintSet::solve
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintSet {
    nodes: [core::cell::Cell<Option<Node>>; ConstraintSet::NODE_CAP],
    nlen: core::cell::Cell<usize>,
    constraints: [core::cell::Cell<Option<Constraint>>; ConstraintSet::CONST_CAP],
    clen: core::cell::Cell<usize>,
    num_vars: usize,
}

impl ConstraintSet {
    const NODE_CAP: usize = 256;
    const CONST_CAP: usize = 64;

    /// Creates an empty constraint set over `num_vars` variables.
    ///
    /// `num_vars` must be one greater than the maximum variable index used;
    /// the solver enumerates over `2^num_vars` boolean assignments.
    #[must_use]
    #[allow(clippy::large_stack_arrays)]
    pub const fn new(num_vars: usize) -> Self {
        Self {
            nodes: [const { core::cell::Cell::new(None) }; Self::NODE_CAP],
            nlen: core::cell::Cell::new(0),
            constraints: [const { core::cell::Cell::new(None) }; Self::CONST_CAP],
            clen: core::cell::Cell::new(0),
            num_vars,
        }
    }

    /// The number of variables this set ranges over.
    #[must_use]
    pub const fn num_vars(&self) -> usize {
        self.num_vars
    }

    /// Pushes a node into the arena, returning its handle, or `None` if the
    /// arena is full.
    fn push_node(&self, node: Node) -> Option<Expr> {
        let len = self.nlen.get();
        if len >= Self::NODE_CAP {
            return None;
        }
        let id = u16::try_from(len).ok()?;
        self.nodes[len].set(Some(node));
        self.nlen.set(len + 1);
        Some(Expr { node: id })
    }

    /// The constant `true` expression.
    ///
    /// # Panics
    ///
    /// Panics if the internal expression arena is exhausted
    /// (`[ConstraintSet::NODE_CAP]` nodes).
    #[must_use]
    pub fn lit_true(&self) -> Expr {
        self.push_node(Node::True).expect("node arena exhausted")
    }

    /// The constant `false` expression.
    ///
    /// # Panics
    ///
    /// Panics if the internal expression arena is exhausted
    /// (`[ConstraintSet::NODE_CAP]` nodes).
    #[must_use]
    pub fn lit_false(&self) -> Expr {
        self.push_node(Node::False).expect("node arena exhausted")
    }

    /// The truth value of `v` as an expression.
    ///
    /// # Panics
    ///
    /// Panics if the internal expression arena is exhausted
    /// (`[ConstraintSet::NODE_CAP]` nodes).
    #[must_use]
    pub fn var(&self, v: Var) -> Expr {
        self.push_node(Node::Var(v)).expect("node arena exhausted")
    }

    /// Logical negation: `not(e)`.
    ///
    /// # Panics
    ///
    /// Panics if the internal expression arena is exhausted
    /// (`[ConstraintSet::NODE_CAP]` nodes).
    #[must_use]
    pub fn not(&self, e: Expr) -> Expr {
        self.push_node(Node::Not(e.node))
            .expect("node arena exhausted")
    }

    /// Logical conjunction: `l and r`.
    ///
    /// # Panics
    ///
    /// Panics if the internal expression arena is exhausted
    /// (`[ConstraintSet::NODE_CAP]` nodes).
    #[must_use]
    pub fn and(&self, l: Expr, r: Expr) -> Expr {
        self.push_node(Node::And(l.node, r.node))
            .expect("node arena exhausted")
    }

    /// Logical disjunction: `l or r`.
    ///
    /// # Panics
    ///
    /// Panics if the internal expression arena is exhausted
    /// (`[ConstraintSet::NODE_CAP]` nodes).
    #[must_use]
    pub fn or(&self, l: Expr, r: Expr) -> Expr {
        self.push_node(Node::Or(l.node, r.node))
            .expect("node arena exhausted")
    }

    /// Logical implication: `l => r` (equivalent to `not(l) or r`).
    ///
    /// # Panics
    ///
    /// Panics if the internal expression arena is exhausted
    /// (`[ConstraintSet::NODE_CAP]` nodes).
    #[must_use]
    pub fn implies(&self, l: Expr, r: Expr) -> Expr {
        self.push_node(Node::Implies(l.node, r.node))
            .expect("node arena exhausted")
    }

    /// Conjunction of all expressions in `exprs`. Returns the constant
    /// `true` for an empty slice (the identity element of `and`).
    #[must_use]
    pub fn all(&self, exprs: &[Expr]) -> Expr {
        let mut acc = self.lit_true();
        for &e in exprs {
            acc = self.and(acc, e);
        }
        acc
    }

    /// Disjunction of all expressions in `exprs`. Returns the constant
    /// `false` for an empty slice (the identity element of `or`).
    #[must_use]
    pub fn any(&self, exprs: &[Expr]) -> Expr {
        let mut acc = self.lit_false();
        for &e in exprs {
            acc = self.or(acc, e);
        }
        acc
    }

    /// Evaluates an expression handle under a boolean assignment.
    ///
    /// `assign[v]` is the truth value of `Var(v)`. A reference to a variable
    /// `>= assign.len()` evaluates to `false`.
    #[must_use]
    pub fn eval(&self, e: Expr, assign: &[bool]) -> bool {
        let Some(node) = self.nodes[e.node as usize].get() else {
            return false;
        };
        match node {
            Node::True => true,
            Node::False => false,
            Node::Var(v) => assign.get(v.0).copied().unwrap_or(false),
            Node::Not(x) => {
                let x = Expr { node: x };
                !self.eval(x, assign)
            }
            Node::And(x, y) => {
                let x = Expr { node: x };
                let y = Expr { node: y };
                self.eval(x, assign) && self.eval(y, assign)
            }
            Node::Or(x, y) => {
                let x = Expr { node: x };
                let y = Expr { node: y };
                self.eval(x, assign) || self.eval(y, assign)
            }
            Node::Implies(x, y) => {
                let x = Expr { node: x };
                let y = Expr { node: y };
                !self.eval(x, assign) || self.eval(y, assign)
            }
        }
    }

    /// Adds a boolean constraint requiring `e` to evaluate to `true`.
    ///
    /// Returns `false` (and leaves the set unchanged) if constraint capacity
    /// is exhausted.
    pub fn add_bool(&self, e: Expr) -> bool {
        self.add(Constraint::Bool(e))
    }

    /// Adds a pseudo-boolean equality `sum(coeff * var) == rhs`.
    ///
    /// Each variable is read as `1` (true) or `0` (false), so this is a
    /// *pseudo-boolean* (0/1) linear equality, not a general integer one.
    ///
    /// Returns `false` (and leaves the set unchanged) if constraint capacity
    /// is exhausted or the term list overflows its inline capacity.
    pub fn pb_eq(&self, pairs: &[(i64, Var)], rhs: i64) -> bool {
        let terms = LinearTerms::from(pairs);
        if terms.len() != pairs.len() {
            return false;
        }
        self.add(Constraint::PbEq { terms, rhs })
    }

    /// Adds a pseudo-boolean inequality `sum(coeff * var) <= rhs`.
    ///
    /// Each variable is read as `1` (true) or `0` (false), so this is a
    /// *pseudo-boolean* (0/1) linear inequality, not a general integer one.
    ///
    /// Returns `false` (and leaves the set unchanged) if constraint capacity
    /// is exhausted or the term list overflows its inline capacity.
    pub fn pb_leq(&self, pairs: &[(i64, Var)], rhs: i64) -> bool {
        let terms = LinearTerms::from(pairs);
        if terms.len() != pairs.len() {
            return false;
        }
        self.add(Constraint::PbLeq { terms, rhs })
    }

    /// Adds a constraint, returning `false` (and leaving the set unchanged)
    /// if capacity is exhausted.
    pub fn add(&self, c: Constraint) -> bool {
        let len = self.clen.get();
        if len >= Self::CONST_CAP {
            return false;
        }
        self.constraints[len].set(Some(c));
        self.clen.set(len + 1);
        true
    }

    /// Iterates over the stored constraints.
    pub fn iter(&self) -> impl Iterator<Item = Constraint> + '_ {
        let clen = self.clen.get();
        let mut i = 0usize;
        core::iter::from_fn(move || {
            while i < clen {
                let idx = i;
                i += 1;
                if let Some(c) = self.constraints[idx].get() {
                    return Some(c);
                }
            }
            None
        })
    }

    /// Solves this constraint set by enumerating all `2^num_vars` boolean
    /// assignments.
    ///
    /// Returns `Some(witness)` for the first satisfying assignment found, or
    /// `None` if the set is unsatisfiable. The set must have at most
    /// `Witness::CAP` variables.
    #[must_use]
    pub fn solve(&self) -> Option<Witness> {
        if self.num_vars > Witness::CAP {
            return None;
        }
        let n = self.num_vars;
        // Enumerating 2^n assignments is only possible when n fits in the
        // machine word: `1usize << n` overflows (and would panic in debug or
        // silently wrap in release) once n >= 64. Such a problem is far too
        // large to enumerate, so report it unsatisfiable-with-respect-to-the
        // bounded solver rather than producing a wrong answer.
        let total = 1usize.checked_shl(u32::try_from(n).unwrap_or(u32::MAX))?;
        for mask in 0..total {
            let mut bools = [false; Witness::CAP];
            for (i, slot) in bools.iter_mut().enumerate().take(n) {
                *slot = (mask >> i) & 1 == 1;
            }
            let witness = Witness {
                bools,
                num_vars: n,
            };
            if self.iter().all(|c| c.holds(self, &bools[..n])) {
                return Some(witness);
            }
        }
        None
    }

    /// Returns `true` if this constraint set is satisfiable.
    #[must_use]
    pub fn satisfiable(&self) -> bool {
        self.solve().is_some()
    }
}

impl Constraint {
    /// Checks whether this constraint holds under a given boolean assignment.
    ///
    /// For [`Constraint::Bool`], `cs` is consulted to evaluate the
    /// expression. For pseudo-boolean constraints, every variable's boolean
    /// assignment is read as `1`/`0` to form the integer sum.
    fn holds(&self, cs: &ConstraintSet, bool_assign: &[bool]) -> bool {
        match self {
            Constraint::Bool(e) => cs.eval(*e, bool_assign),
            Constraint::PbEq { terms, rhs } => eval_linear(terms, bool_assign) == *rhs,
            Constraint::PbLeq { terms, rhs } => eval_linear(terms, bool_assign) <= *rhs,
        }
    }
}

/// Evaluates a list of pseudo-boolean terms to an `i64`, treating each boolean
/// variable as `1` (true) or `0` (false).
fn eval_linear(terms: &LinearTerms, bool_assign: &[bool]) -> i64 {
    let mut sum: i64 = 0;
    for term in terms.iter() {
        let value: i64 = i64::from(bool_assign.get(term.var.0).copied().unwrap_or(false));
        sum = sum.saturating_add(term.coeff.saturating_mul(value));
    }
    sum
}

/// A satisfying assignment found by the solver.
///
/// Boolean variables always carry both a truth value (the enumerated
/// assignment) and an integer value (`1`/`0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Witness {
    bools: [bool; Witness::CAP],
    num_vars: usize,
}

impl Witness {
    const CAP: usize = 64;

    /// The truth value of `v` under this witness.
    #[must_use]
    pub fn bool_of(&self, v: Var) -> Option<bool> {
        if v.0 < self.num_vars {
            Some(self.bools[v.0])
        } else {
            None
        }
    }

    /// The integer value of `v` under this witness (`1` if true, `0` if
    /// false).
    #[must_use]
    pub fn int_of(&self, v: Var) -> Option<i64> {
        self.bool_of(v).map(i64::from)
    }

    /// The underlying boolean assignment slice.
    #[must_use]
    pub fn bools(&self) -> &[bool] {
        &self.bools[..self.num_vars]
    }
}

/// Solves a [`ConstraintSet`] by enumerating all `2^num_vars` boolean
/// assignments.
///
/// Returns `Some(witness)` for the first satisfying assignment found, or
/// `None` if the set is unsatisfiable.
///
/// ```
/// use tpt_zero_smt_lite::{Var, ConstraintSet, solve};
///
/// let a = Var(0);
/// let b = Var(1);
/// let cs = ConstraintSet::new(2);
/// cs.add_bool(cs.and(cs.or(cs.var(a), cs.var(b)), cs.not(cs.var(a))));
/// cs.pb_leq(&[(1, a), (1, b)], 1);
/// assert!(solve(&cs).is_some());
/// ```
#[must_use]
pub fn solve(cs: &ConstraintSet) -> Option<Witness> {
    cs.solve()
}

/// Returns `true` if the constraint set is satisfiable.
///
/// Convenience wrapper around [`solve`].
#[must_use]
pub fn satisfiable(cs: &ConstraintSet) -> bool {
    cs.satisfiable()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expr_eval_matches_core_semantics() {
        let a = Var(0);
        let b = Var(1);
        let cs = ConstraintSet::new(2);
        let assign = [true, false];
        assert!(cs.eval(cs.var(a), &assign));
        assert!(!cs.eval(cs.var(b), &assign));
        assert!(!cs.eval(cs.not(cs.var(a)), &assign));
        assert!(cs.eval(cs.and(cs.var(a), cs.not(cs.var(b))), &assign));
        assert!(cs.eval(cs.or(cs.var(a), cs.var(b)), &assign));
        assert!(cs.eval(cs.implies(cs.var(b), cs.var(a)), &assign));
        assert!(!cs.eval(cs.implies(cs.var(a), cs.var(b)), &assign));
        assert!(cs.eval(cs.lit_true(), &assign));
        assert!(!cs.eval(cs.lit_false(), &assign));
    }

    #[test]
    fn all_and_any_identity() {
        let cs = ConstraintSet::new(0);
        let empty: [Expr; 0] = [];
        assert!(cs.eval(cs.all(&empty), &[]));
        assert!(!cs.eval(cs.any(&empty), &[]));
    }

    #[test]
    fn boolean_sat_finds_witness() {
        let a = Var(0);
        let b = Var(1);
        let cs = ConstraintSet::new(2);
        cs.add_bool(cs.and(cs.or(cs.var(a), cs.var(b)), cs.not(cs.var(a))));
        let w = cs.solve().expect("satisfiable");
        assert_eq!(w.bool_of(a), Some(false));
        assert_eq!(w.bool_of(b), Some(true));
    }

    #[test]
    fn unsatisfiable_when_a_and_not_a() {
        let a = Var(0);
        let cs = ConstraintSet::new(1);
        cs.add_bool(cs.and(cs.var(a), cs.not(cs.var(a))));
        assert!(cs.solve().is_none());
        assert!(!cs.satisfiable());
    }

    #[test]
    fn linear_eq_over_bools() {
        let x = Var(0);
        let cs = ConstraintSet::new(1);
        // 2x == 10 has no boolean (0/1) solution.
        cs.pb_eq(&[(2, x)], 10);
        assert!(cs.solve().is_none());
    }

    #[test]
    fn linear_leq_sat_over_booleans() {
        let x = Var(0);
        let y = Var(1);
        let cs = ConstraintSet::new(2);
        cs.pb_leq(&[(1, x), (1, y)], 1);
        assert!(cs.solve().is_some());
        let cs2 = ConstraintSet::new(2);
        cs2.pb_leq(&[(1, x), (1, y)], -1);
        assert!(cs2.solve().is_none());
    }

    #[test]
    fn constraint_capacity_respected() {
        let cs = ConstraintSet::new(1);
        for _ in 0..ConstraintSet::CONST_CAP {
            assert!(cs.add(Constraint::Bool(cs.lit_true())));
        }
        assert!(!cs.add(Constraint::Bool(cs.lit_true())));
    }

    #[test]
    fn linear_terms_from_pairs() {
        let x = Var(0);
        let y = Var(1);
        let pairs = [(2i64, x), (-1i64, y), (2i64, x)];
        let terms = LinearTerms::from(pairs.as_slice());
        // 2x - y + 2x = 4x - y; under (x=1,y=1) => 3.
        let assign = [true, true];
        assert_eq!(eval_linear(&terms, &assign), 3);
    }

    #[test]
    fn too_many_vars_returns_none() {
        let cs = ConstraintSet::new(Witness::CAP + 1);
        assert!(cs.solve().is_none());
    }

    #[test]
    fn de_morgan_via_set() {
        let a = Var(0);
        let b = Var(1);
        let cs = ConstraintSet::new(2);
        let assign = [true, false];
        let lhs = cs.not(cs.and(cs.var(a), cs.var(b)));
        let rhs = cs.or(cs.not(cs.var(a)), cs.not(cs.var(b)));
        assert_eq!(cs.eval(lhs, &assign), cs.eval(rhs, &assign));
    }

    mod proptest_boolean {
        use super::*;
        use proptest::arbitrary::any;
        use proptest::prelude::*;

        /// An independent, heap-free reference representation of a boolean
        /// expression used to cross-check [`ConstraintSet::eval`].
        #[derive(Clone, Copy)]
        #[allow(dead_code)]
        enum RefExpr {
            True,
            False,
            Var(Var),
            Not(ExprRef),
            And(ExprRef, ExprRef),
            Or(ExprRef, ExprRef),
            Implies(ExprRef, ExprRef),
        }

        #[derive(Clone, Copy)]
        #[allow(dead_code)]
        struct ExprRef(usize);

        /// Builds a random reference expression and, in lock-step, the
        /// equivalent [`Expr`] tree in `cs`.
        struct Builder<'a> {
            cs: &'a ConstraintSet,
            refs: [Option<RefExpr>; Builder::CAP],
            exprs: [Option<Expr>; Builder::CAP],
            len: usize,
        }

        impl<'a> Builder<'a> {
            const CAP: usize = 64;

            fn new(cs: &'a ConstraintSet) -> Self {
                Self {
                    cs,
                    refs: [None; Self::CAP],
                    exprs: [None; Self::CAP],
                    len: 0,
                }
            }

            fn push(&mut self, r: RefExpr) -> Option<ExprRef> {
                if self.len >= Self::CAP {
                    return None;
                }
                let id = self.len;
                self.refs[self.len] = Some(r);
                self.len += 1;
                Some(ExprRef(id))
            }

            fn leaf(&mut self, nvars: usize) -> ExprRef {
                let v = Var(self.len % nvars.max(1));
                let r = self.push(RefExpr::Var(v)).expect("ref arena exhausted");
                let e = self.cs.var(v);
                // Keep the indices aligned: map ref id -> expr.
                self.exprs[r.0] = Some(e);
                r
            }

            fn build(&mut self, depth: usize, nvars: usize) -> ExprRef {
                if depth == 0 {
                    return self.leaf(nvars);
                }
                let l = self.build(depth - 1, nvars);
                let r = self.build(depth - 1, nvars);
                let kind = self.len % 4;
                let rnode = match kind {
                    0 => RefExpr::Not(l),
                    1 => RefExpr::And(l, r),
                    2 => RefExpr::Or(l, r),
                    _ => RefExpr::Implies(l, r),
                };
                let ref_id = self.push(rnode).expect("ref arena exhausted");
                let e = match kind {
                    0 => self.cs.not(self.expr_of(l)),
                    1 => self.cs.and(self.expr_of(l), self.expr_of(r)),
                    2 => self.cs.or(self.expr_of(l), self.expr_of(r)),
                    _ => self.cs.implies(self.expr_of(l), self.expr_of(r)),
                };
                self.exprs[ref_id.0] = Some(e);
                ref_id
            }

            fn expr_of(&self, r: ExprRef) -> Expr {
                self.exprs[r.0].expect("expr not built")
            }

            fn eval_ref(&self, r: ExprRef, assign: &[bool]) -> bool {
                match self.refs[r.0].expect("ref not built") {
                    RefExpr::True => true,
                    RefExpr::False => false,
                    RefExpr::Var(v) => assign.get(v.0).copied().unwrap_or(false),
                    RefExpr::Not(x) => !self.eval_ref(x, assign),
                    RefExpr::And(a, b) => self.eval_ref(a, assign) && self.eval_ref(b, assign),
                    RefExpr::Or(a, b) => self.eval_ref(a, assign) || self.eval_ref(b, assign),
                    RefExpr::Implies(a, b) => !self.eval_ref(a, assign) || self.eval_ref(b, assign),
                }
            }
        }

        proptest! {
            #[test]
            fn eval_matches_reference(
                a in any::<bool>(),
                b in any::<bool>(),
                c in any::<bool>(),
            ) {
                let assign = [a, b, c];
                let cs = ConstraintSet::new(3);
                let mut builder = Builder::new(&cs);
                let root = builder.build(3, 3);
                let expected = builder.eval_ref(root, &assign);
                let root_expr = builder.expr_of(root);
                let got = cs.eval(root_expr, &assign);
                prop_assert_eq!(got, expected);
            }

            #[test]
            fn de_morgan(a in any::<bool>(), b in any::<bool>()) {
                let assign = [a, b];
                let cs = ConstraintSet::new(2);
                let lhs = cs.not(cs.and(cs.var(Var(0)), cs.var(Var(1))));
                let rhs = cs.or(cs.not(cs.var(Var(0))), cs.not(cs.var(Var(1))));
                prop_assert_eq!(cs.eval(lhs, &assign), cs.eval(rhs, &assign));
            }

            #[test]
            fn imply_equiv(a in any::<bool>(), b in any::<bool>()) {
                let assign = [a, b];
                let cs = ConstraintSet::new(2);
                let lhs = cs.implies(cs.var(Var(0)), cs.var(Var(1)));
                let rhs = cs.or(cs.not(cs.var(Var(0))), cs.var(Var(1)));
                prop_assert_eq!(cs.eval(lhs, &assign), cs.eval(rhs, &assign));
            }

            #[test]
            fn double_negation(a in any::<bool>()) {
                let assign = [a];
                let cs = ConstraintSet::new(1);
                let e = cs.not(cs.not(cs.var(Var(0))));
                prop_assert_eq!(cs.eval(e, &assign), a);
            }
        }
    }
}
