#![doc = include_str!("../README.md")]
#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "assert-const")]
pub use out_zero_assert_const as assert_const;
#[cfg(feature = "bounded")]
pub use out_zero_bounded as bounded;
#[cfg(feature = "contract")]
pub use out_zero_contract as contract;
#[cfg(feature = "float")]
pub use tpt_zero_float as float;
#[cfg(feature = "loop-inv")]
pub use out_zero_loop_inv as loop_inv;
#[cfg(feature = "phantom")]
pub use out_zero_phantom as phantom;
#[cfg(feature = "postcond")]
pub use out_zero_postcond as postcond;
#[cfg(feature = "precond")]
pub use out_zero_precond as precond;
#[cfg(feature = "refinement")]
pub use out_zero_refinement as refinement;
#[cfg(feature = "safe-cast")]
pub use out_zero_safe_cast as safe_cast;
#[cfg(feature = "type-level")]
pub use out_zero_type_level as type_level;
#[cfg(feature = "bayes")]
pub use tpt_zero_bayes::{Beta, Gamma};
#[cfg(feature = "decomp")]
pub use tpt_zero_decomp as decomp;
#[cfg(feature = "dist")]
pub use tpt_zero_dist as dist;
#[cfg(feature = "dist")]
pub use tpt_zero_dist::Distribution;
#[cfg(feature = "eigen")]
pub use tpt_zero_eigen as eigen;
#[cfg(feature = "fsm")]
pub use tpt_zero_fsm as fsm;
#[cfg(feature = "ghost")]
pub use tpt_zero_ghost as ghost;
#[cfg(feature = "grad")]
pub use tpt_zero_grad as grad;
#[cfg(feature = "invariant")]
pub use tpt_zero_invariant as invariant;
#[cfg(feature = "linalg")]
pub use tpt_zero_linalg as linalg;
#[cfg(feature = "markov")]
pub use tpt_zero_markov as markov;
#[cfg(feature = "monte-carlo")]
pub use tpt_zero_monte_carlo as monte_carlo;
#[cfg(feature = "prob")]
pub use tpt_zero_prob as prob;
#[cfg(feature = "rand")]
pub use tpt_zero_rand as rand;
#[cfg(feature = "sampler")]
pub use tpt_zero_sampler as sampler;
#[cfg(feature = "smt-lite")]
pub use tpt_zero_smt_lite as smt_lite;
#[cfg(feature = "solver")]
pub use tpt_zero_solver as solver;
#[cfg(feature = "stats")]
pub use tpt_zero_stats as stats;
#[cfg(feature = "tensor")]
pub use tpt_zero_tensor as tensor;
#[cfg(feature = "witness")]
pub use tpt_zero_witness as witness;

/// Commonly used, unambiguous items. Import with `use out_zero_formal::prelude::*;`.
pub mod prelude {
    #[cfg(feature = "bounded")]
    pub use out_zero_bounded::BoundedInt;
    #[cfg(feature = "float")]
    pub use tpt_zero_float::{exp, ln, sqrt};
    #[cfg(feature = "bayes")]
    pub use tpt_zero_bayes::{Beta, Gamma};
    #[cfg(feature = "dist")]
    pub use tpt_zero_dist::{Bernoulli, Normal, Poisson, Uniform};
    #[cfg(feature = "ghost")]
    pub use tpt_zero_ghost::{Ghost, GhostProven, Proven, Unproven};
    #[cfg(feature = "prob")]
    pub use tpt_zero_prob::{Dist, Distribution};
    #[cfg(feature = "rand")]
    pub use tpt_zero_rand::Rng;
    #[cfg(feature = "tensor")]
    pub use tpt_zero_tensor::{Tensor, Tensor2};
    #[cfg(feature = "witness")]
    pub use tpt_zero_witness::Witness;
}
