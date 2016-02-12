//! Monte-Carlo simulation of a Xenon crystal melt.
extern crate cymbalum;
use cymbalum::*;

fn main() {
    Logger::stdout();

    let mut universe = Universe::from_file("data/NaCl.xyz").unwrap();
    universe.set_cell(UnitCell::cubic(units::from(21.65, "A").unwrap()));

    universe.add_pair_interaction("Xe", "Xe",
        Box::new(LennardJones{
            sigma: units::from(4.57, "A").unwrap(),
            epsilon: units::from(1.87, "kJ/mol").unwrap()
        }
    ));

    // Create a Monte-Carlo propagator
    let mut mc = MonteCarlo::new(units::from(500.0, "K").unwrap());
    // Add the `Translate` move with 0.5 A amplitude and 1.0 frequency
    mc.add(
        Box::new(Translate::new(units::from(0.5, "A").unwrap())),
        1.0
    );
    let mut simulation = Simulation::new(mc);
    simulation.add_output_with_frequency(TrajectoryOutput::new("trajectory.xyz").unwrap(), 50);

    simulation.run(&mut universe, 20000);
}