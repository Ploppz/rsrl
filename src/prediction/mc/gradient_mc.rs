use crate::core::*;
use crate::domains::Transition;
use crate::fa::{Parameterised, VFunction};

pub struct GradientMC<V> {
    pub v_func: Shared<V>,

    pub alpha: Parameter,
    pub gamma: Parameter,
}

impl<V> GradientMC<V> {
    pub fn new<T1, T2>(v_func: Shared<V>, alpha: T1, gamma: T2) -> Self
    where
        T1: Into<Parameter>,
        T2: Into<Parameter>,
    {
        GradientMC {
            v_func,

            alpha: alpha.into(),
            gamma: gamma.into(),
        }
    }
}

impl<V> Algorithm for GradientMC<V> {
    fn handle_terminal(&mut self) {
        self.alpha = self.alpha.step();
        self.gamma = self.gamma.step();
    }
}

impl<S, A, V: VFunction<S>> BatchLearner<S, A> for GradientMC<V> {
    fn handle_batch(&mut self, batch: &[Transition<S, A>]) {
        let mut sum = 0.0;

        batch.into_iter().rev().for_each(|ref t| {
            sum = t.reward + self.gamma * sum;

            let s = t.from.state();
            let v_est = self.v_func.evaluate(s).unwrap();
            let _ = self.v_func.borrow_mut().update(s, self.alpha * (sum - v_est));
        })
    }
}

impl<S, V: VFunction<S>> ValuePredictor<S> for GradientMC<V> {
    fn predict_v(&mut self, s: &S) -> f64 {
        self.v_func.evaluate(s).unwrap()
    }
}

impl<S, A, V: VFunction<S>> ActionValuePredictor<S, A> for GradientMC<V> {}

impl<V: Parameterised> Parameterised for GradientMC<V> {
    fn weights(&self) -> Matrix<f64> {
        self.v_func.weights()
    }
}
