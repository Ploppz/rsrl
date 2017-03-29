use super::Agent;

use {Function, Parameterised};
use domain::Transition;
use geometry::{Space, ActionSpace};
use policies::{Policy, Greedy};

use std::fmt::Debug;


pub struct QLearning<Q, P> {
    q_func: Q,

    exploration_policy: P,
    greedy_policy: Greedy,

    alpha: f64,
    gamma: f64,
}

impl<Q, P> QLearning<Q, P> {
    pub fn new(q_func: Q, policy: P) -> Self {
        QLearning {
            q_func: q_func,

            exploration_policy: policy,
            greedy_policy: Greedy,

            alpha: 0.10,
            gamma: 0.95,
        }
    }
}

impl<S: Space, Q, P: Policy> Agent<S> for QLearning<Q, P>
    where Q: Function<S::Repr, Vec<f64>> + Parameterised<S::Repr, Vec<f64>>,
          S::Repr: Debug
{
    fn handle(&mut self, t: &Transition<S, ActionSpace>) -> usize {
        let (s, ns) = (t.from.get(), t.to.get());

        let qs = self.q_func.evaluate(s);
        let nqs = self.q_func.evaluate(ns);

        let a = t.action;
        let na = self.greedy_policy.sample(nqs.as_slice());

        let mut errors = vec![0.0; qs.len()];
        errors[a] = self.alpha*(t.reward + self.gamma*nqs[na] - qs[a]);

        self.q_func.update(s, &errors);

        self.exploration_policy.sample(qs.as_slice())
    }
}


pub struct SARSA<Q, P> {
    q_func: Q,
    policy: P,

    alpha: f64,
    gamma: f64,
}

impl<Q, P> SARSA<Q, P> {
    pub fn new(q_func: Q, policy: P) -> Self {
        SARSA {
            q_func: q_func,
            policy: policy,

            alpha: 0.10,
            gamma: 0.95,
        }
    }
}

impl<S: Space, Q, P: Policy> Agent<S> for SARSA<Q, P>
    where Q: Function<S::Repr, Vec<f64>> + Parameterised<S::Repr, Vec<f64>>,
          S::Repr: Debug
{
    fn handle(&mut self, t: &Transition<S, ActionSpace>) -> usize {
        let (s, ns) = (t.from.get(), t.to.get());

        let qs = self.q_func.evaluate(s);
        let nqs = self.q_func.evaluate(ns);

        let a = t.action;
        let na = self.policy.sample(nqs.as_slice());

        let mut errors = vec![0.0; qs.len()];
        errors[a] = self.alpha*(t.reward + self.gamma*nqs[na] - qs[a]);

        self.q_func.update(s, &errors);

        self.policy.sample(qs.as_slice())
    }
}