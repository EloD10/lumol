// Cymbalum, an extensible molecular simulation engine
// Copyright (C) 2015-2016 G. Fraux — BSD license

//! Wolf summation of coulombic potential
extern crate special;
use self::special::erfc;

use std::f64::consts::PI;

use system::System;
use types::{Matrix3, Vector3D, Zero};
use constants::ELCC;
use potentials::PairRestriction;

use super::{GlobalPotential, CoulombicPotential, DefaultGlobalCache};

/// Wolf summation for the coulombic potential, as defined in [Wolf1999]. This
/// is a fast direct pairwise summation for coulombic potential.
///
/// [Wolf1999]: Wolf, D. et al. J. Chem. Phys. 110, 8254 (1999).
#[derive(Clone)]
pub struct Wolf {
    /// Spliting parameter
    alpha: f64,
    /// Cutoff radius in real space
    cutoff: f64,
    /// Energy constant for wolf computation
    energy_cst: f64,
    /// Force constant for wolf computation
    force_cst: f64,
    /// Restriction scheme
    restriction: PairRestriction,
}

impl Wolf {
    /// Create a new Wolf summation, using a real-space cutoff of `cutoff`.
    pub fn new(cutoff: f64) -> Wolf {
        assert!(cutoff > 0.0, "Got a negative cutoff in Wolf summation");
        let alpha = PI/cutoff;
        let e_cst = erfc(alpha*cutoff)/cutoff;
        let f_cst = erfc(alpha*cutoff)/(cutoff*cutoff) + 2.0*alpha/f64::sqrt(PI) * f64::exp(-alpha*alpha*cutoff*cutoff)/cutoff;
        Wolf{
            alpha: alpha,
            cutoff: cutoff,
            energy_cst: e_cst,
            force_cst: f_cst,
            restriction: PairRestriction::None,
        }
    }

    /// Compute the energy for the pair of particles with charge `qi` and `qj`,
    /// at the distance of `rij`.
    #[inline]
    fn energy_pair(&self, qi: f64, qj: f64, rij: f64) -> f64 {
        if rij > self.cutoff {
            return 0.0;
        }
        qi * qj * (erfc(self.alpha*rij)/rij - self.energy_cst) / ELCC
    }

    /// Compute the energy for self interaction of a particle with charge `qi`
    #[inline]
    fn energy_self(&self, qi: f64) -> f64 {
        qi * qi * (self.energy_cst/2.0 + self.alpha/f64::sqrt(PI)) / ELCC
    }

    /// Compute the force for self the pair of particles with charge `qi` and
    /// `qj`, at the distance of `rij`.
    #[inline]
    fn force_pair(&self, qi: f64, qj: f64, rij: Vector3D) -> Vector3D {
        let d = rij.norm();
        if d > self.cutoff {
            return Vector3D::new(0.0, 0.0, 0.0);
        }
        let factor = erfc(self.alpha*d)/(d*d) + 2.0*self.alpha/f64::sqrt(PI) * f64::exp(-self.alpha*self.alpha*d*d)/d;
        return qi * qj * (factor - self.force_cst) * rij.normalized() / ELCC;
    }
}

impl GlobalPotential for Wolf {
    fn energy(&self, system: &System) -> f64 {
        let natoms = system.size();
        let mut res = 0.0;
        for i in 0..natoms {
            let qi = system[i].charge;
            for j in i+1..natoms {
                let qj = system[j].charge;
                if qi*qj == 0.0 {
                    continue;
                }
                if !self.restriction.is_excluded_pair(system, i, j) {
                    let s = self.restriction.scaling(system, i, j);
                    let rij = system.distance(i, j);
                    res += s * self.energy_pair(qi, qj, rij);
                }
            }
            // Remove self term
            res -= self.energy_self(qi);
        }
        return res;
    }

    fn forces(&self, system: &System) -> Vec<Vector3D> {
        let natoms = system.size();
        let mut res = vec![Vector3D::new(0.0, 0.0, 0.0); natoms];
        for i in 0..natoms {
            let qi = system[i].charge;
            for j in i+1..natoms {
                let qj = system[j].charge;
                if qi*qj == 0.0 {
                    continue;
                }
                if !self.restriction.is_excluded_pair(system, i, j) {
                    let s = self.restriction.scaling(system, i, j);
                    let rij = system.wraped_vector(i, j);
                    let force = s * self.force_pair(qi, qj, rij);
                    res[i] = res[i] + force;
                    res[j] = res[j] - force;
                }
            }
        }
        return res;
    }

    fn virial(&self, system: &System) -> Matrix3 {
        let natoms = system.size();
        let mut res = Matrix3::zero();
        for i in 0..natoms {
            let qi = system[i].charge;
            for j in i+1..natoms {
                let qj = system[j].charge;
                if qi*qj == 0.0 {
                    continue;
                }
                if !self.restriction.is_excluded_pair(system, i, j) {
                    let s = self.restriction.scaling(system, i, j);
                    let rij = system.wraped_vector(i, j);
                    let force = s * self.force_pair(qi, qj, rij);
                    res = res + force.tensorial(&rij);
                }
            }
        }
        return res;
    }
}

impl CoulombicPotential for Wolf {
    fn set_restriction(&mut self, restriction: PairRestriction) {
        self.restriction = restriction;
    }
}

impl DefaultGlobalCache for Wolf {}

#[cfg(test)]
mod tests {
    pub use super::*;
    use system::{System, UnitCell, Particle};
    use types::Vector3D;
    use potentials::GlobalPotential;

    const E_BRUTE_FORCE: f64 = -0.09262397663346732;

    pub fn testing_system() -> System {
        let mut system = System::from_cell(UnitCell::cubic(20.0));

        system.add_particle(Particle::new("Cl"));
        system[0].charge = -1.0;
        system[0].position = Vector3D::new(0.0, 0.0, 0.0);

        system.add_particle(Particle::new("Na"));
        system[1].charge = 1.0;
        system[1].position = Vector3D::new(1.5, 0.0, 0.0);

        return system;
    }

    #[test]
    fn energy() {
        let system = testing_system();
        let wolf = Wolf::new(8.0);

        let e = wolf.energy(&system);
        // Wolf is not very good for inhomogeneous systems
        assert_approx_eq!(e, E_BRUTE_FORCE, 1e-2);
    }

    #[test]
    fn forces() {
        let mut system = testing_system();
        let wolf = Wolf::new(8.0);

        let forces = wolf.forces(&system);
        let norm = (forces[0] + forces[1]).norm();
        // Total force should be null
        assert_approx_eq!(norm, 0.0, 1e-9);

        // Finite difference computation of the force
        let e = wolf.energy(&system);
        let eps = 1e-9;
        system[0].position.x += eps;

        let e1 = wolf.energy(&system);
        let force = wolf.forces(&system)[0].x;
        assert_approx_eq!((e - e1)/eps, force, 1e-6);
    }
}
