//! Function approximation and value function representation module.
use crate::core::Shared;
use crate::geometry::Vector;

extern crate lfa;
pub use self::lfa::{
    approximators::*,
    basis::{
        self,
        Projector,
        Projection,
    },
    core::{
        AdaptResult,
        EvaluationResult,
        UpdateResult,
        Parameterised,
    },
    LFA,
};

#[cfg(test)]
pub(crate) mod mocking;

mod table;
pub use self::table::Table;

pub type ScalarLFA<P> = LFA<P, ScalarFunction>;
pub type VectorLFA<P> = LFA<P, VectorFunction>;

pub type SharedVFunction<S> = Shared<VFunction<S, Value = f64>>;
pub type SharedQFunction<S> = Shared<QFunction<S, Value = Vector<f64>>>;

/// An interface for state-value functions.
pub trait VFunction<S: ?Sized>: Approximator<S, Value = f64> {
    #[allow(unused_variables)]
    fn evaluate_phi(&self, phi: &Projection) -> f64 { unimplemented!() }

    #[allow(unused_variables)]
    fn update_phi(&mut self, phi: &Projection, update: f64) { unimplemented!() }
}

impl<S: ?Sized, P: Projector<S>> VFunction<S> for ScalarLFA<P> {
    fn evaluate_phi(&self, phi: &Projection) -> f64 {
        self.evaluate_primal(phi).unwrap()
    }

    fn update_phi(&mut self, phi: &Projection, update: f64) {
        let _ = self.update_primal(phi, update);
    }
}

/// An interface for action-value functions.
pub trait QFunction<S: ?Sized>: Approximator<S, Value = Vector<f64>> {
    fn evaluate_action(&self, input: &S, action: usize) -> f64 {
        self.evaluate(input).unwrap()[action]
    }

    #[allow(unused_variables)]
    fn update_action(&mut self, input: &S, action: usize, update: f64) { unimplemented!() }

    #[allow(unused_variables)]
    fn evaluate_phi(&self, phi: &Projection) -> Vector<f64> { unimplemented!() }

    #[allow(unused_variables)]
    fn evaluate_action_phi(&self, phi: &Projection, action: usize) -> f64 { unimplemented!() }

    #[allow(unused_variables)]
    fn update_phi(&mut self, phi: &Projection, updates: Vector<f64>) { unimplemented!() }

    #[allow(unused_variables)]
    fn update_action_phi(&mut self, phi: &Projection, action: usize, update: f64) {
        unimplemented!()
    }
}

impl<S: ?Sized, P: Projector<S>> QFunction<S> for VectorLFA<P> {
    fn evaluate_action(&self, input: &S, action: usize) -> f64 {
        let p = self.projector.project(input);

        self.evaluate_action_phi(&p, action)
    }

    fn update_action(&mut self, input: &S, action: usize, update: f64) {
        let p = self.projector.project(input);

        self.update_action_phi(&p, action, update);
    }

    fn evaluate_phi(&self, phi: &Projection) -> Vector<f64> {
        self.approximator.evaluate(&phi).unwrap()
    }

    fn evaluate_action_phi(&self, phi: &Projection, action: usize) -> f64 {
        let col = self.approximator.weights.column(action);

        match *phi {
            Projection::Dense(ref dense) => col.dot(dense),
            Projection::Sparse(ref sparse) => sparse.iter().fold(0.0, |acc, idx| acc + col[*idx]),
        }
    }

    fn update_phi(&mut self, phi: &Projection, updates: Vector<f64>) {
        let _ = self.approximator.update(phi, updates);
    }

    fn update_action_phi(&mut self, phi: &Projection, action: usize, update: f64) {
        let mut col = self.approximator.weights.column_mut(action);

        match *phi {
            Projection::Dense(ref dense) => col.scaled_add(update, dense),
            Projection::Sparse(ref sparse) => {
                for idx in sparse {
                    col[*idx] += update
                }
            },
        }
    }
}
