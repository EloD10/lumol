// Lumol, an extensible molecular simulation engine
// Copyright (C) 2015-2016 G. Fraux — BSD license

use super::{Potential, PairPotential};

/// Methods for energy and forces computation.
///
/// A potential computation is a way of computing a potential given its
/// expression (represented by a `Potential`). The same potential can be
/// computed either direcly, or using a cutoff, or by a table interpolation, ...
pub trait Computation: Sync + Send {
    /// Compute the energy value at `r`
    fn compute_energy(&self, r: f64) -> f64;

    /// Compute the force value at `r`
    fn compute_force(&self, r: f64) -> f64;
}

impl<P: Computation + Clone + 'static> Potential for P {
    #[inline] fn energy(&self, r:f64) -> f64 {
        self.compute_energy(r)
    }

    #[inline] fn force(&self, r:f64) -> f64 {
        self.compute_force(r)
    }
}

/// Computation of a potential using tabulated values.
///
/// This can be faster than direct computation for smooth potentials, but will
/// either uses more memory or be less precise than direct computation. Values
/// are tabulated in the `[0, max)` range, and a cutoff is applied after `max`.
/// Energy is shifted to ensure `E(max) = 0`
#[derive(Clone)]
pub struct TableComputation {
    // TODO: use genericity over static values here if it ever comes out
    // see https://internals.rust-lang.org/t/pre-rfc-genericity-over-static-values/1538/19

    /// Number of tabulated values
    size: usize,
    /// Step for tabulated value. `energy_table[i]`/`force_table[i]` contains
    /// energy/force at `r = i * delta`
    delta: f64,
    /// Tabulated potential
    energy_table: Vec<f64>,
    /// Tabulated compute_force
    force_table: Vec<f64>,
}


impl TableComputation {
    /// Create a new `TableComputation` for `potential`, with `size` points and
    /// a maximum value of `max`.
    pub fn new(potential: Box<PairPotential>, size: usize, max:f64) -> TableComputation {
        let delta = max/(size as f64);
        let energy_shift = potential.energy(max);
        let mut energy_table = Vec::with_capacity(size);
        let mut force_table = Vec::with_capacity(size);
        for i in 0..size {
            let pos = i as f64 * delta;
            energy_table.push(potential.energy(pos) - energy_shift);
            force_table.push(potential.force(pos));
        }
        TableComputation {
            size: size,
            delta: delta,
            energy_table: energy_table,
            force_table: force_table
        }
    }
}

impl Computation for TableComputation {
    fn compute_energy(&self, r: f64) -> f64 {
        let bin = f64::floor(r / self.delta) as usize;
        if bin < self.size - 1 {
            let dx = r - (bin as f64)*self.delta;
            let slope = (self.energy_table[bin + 1] - self.energy_table[bin])/self.delta;
            return self.energy_table[bin] + dx*slope;
        } else {
            return 0.0;
        }
    }

    fn compute_force(&self, r: f64) -> f64 {
        let bin = f64::floor(r / self.delta) as usize;
        if bin < self.size - 1 {
            let dx = r - (bin as f64)*self.delta;
            let slope = (self.force_table[bin + 1] - self.force_table[bin])/self.delta;
            return self.force_table[bin] + dx*slope;
        } else {
            return 0.0;
        }
    }
}

impl PairPotential for TableComputation {}

/******************************************************************************/

#[cfg(test)]
mod test {
    use super::*;
    use energy::Harmonic;

    #[test]
    fn table() {
        let table = TableComputation::new(Box::new(Harmonic{k: 50.0, x0: 2.0}), 1000, 4.0);

        assert_eq!(table.compute_force(2.5), -25.0);
        assert_eq!(table.compute_energy(2.5), -93.75);

        assert_eq!(table.compute_force(4.1), 0.0);
        assert_eq!(table.compute_energy(4.1), 0.0);
    }
}
