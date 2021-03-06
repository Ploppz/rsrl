extern crate rsrl;
#[macro_use]
extern crate slog;

use rsrl::{
    control::gtd::GreedyGQ,
    core::{make_shared, run, Evaluation, Parameter, SerialExperiment},
    domains::{Domain, MountainCar},
    fa::{basis::fixed::Fourier, LFA},
    geometry::Space,
    logging,
    policies::fixed::{Greedy, Random, EpsilonGreedy},
};

fn main() {
    let logger = logging::root(logging::stdout());

    let domain = MountainCar::default();
    let mut agent = {
        let n_actions = domain.action_space().card().into();

        // Build the linear value functions using a fourier basis projection.
        let bases = Fourier::from_space(3, domain.state_space());
        let v_func = make_shared(LFA::scalar_output(bases.clone()));
        let q_func = make_shared(LFA::vector_output(bases, n_actions));

        // Build a stochastic behaviour policy with exponential epsilon.
        let policy = make_shared(EpsilonGreedy::new(
            Greedy::new(q_func.clone()),
            Random::new(n_actions),
            Parameter::exponential(0.3, 0.001, 0.99),
        ));

        GreedyGQ::new(q_func, v_func, policy, 1e-3, 1e-4, 0.99)
    };

    let domain_builder = Box::new(MountainCar::default);

    // Training phase:
    let _training_result = {
        // Start a serial learning experiment up to 1000 steps per episode.
        let e = SerialExperiment::new(&mut agent, domain_builder.clone(), 1000);

        // Realise 1000 episodes of the experiment generator.
        run(e, 2000, Some(logger.clone()))
    };

    // Testing phase:
    let testing_result = Evaluation::new(&mut agent, domain_builder).next().unwrap();

    info!(logger, "solution"; testing_result);
}
