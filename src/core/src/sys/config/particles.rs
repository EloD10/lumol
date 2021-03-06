// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

//! `Particle` type and manipulation.
use types::{Vector3D, Zero};
use sys::PeriodicTable;

use std::fmt;

/// A particle kind. Particles with the same name will have the same kind. This
/// is used for faster potential lookup.
#[derive(Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct ParticleKind(pub u32);

impl ParticleKind {
    /// Get an invalid value (`u32::max_value()`) to use as a marker
    pub fn invalid() -> ParticleKind {
        ParticleKind(u32::max_value())
    }
}

impl fmt::Display for ParticleKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The Particle type hold basic data about a particle in the system. It is self
/// contained, so that it will be easy to send data between parallels processes.
#[derive(Clone, Debug)]
pub struct Particle {
    /// Particle name. This one is not public, as we always want to get &str,
    /// and to use either `String` of `&str` to set it.
    name: String,
    /// Particle kind, an index for potentials lookup
    pub(in ::sys) kind: ParticleKind,
    /// Particle mass
    pub mass: f64,
    /// Particle charge
    pub charge: f64,
    /// Particle positions
    pub position: Vector3D,
    /// Particle velocity, if needed
    pub velocity: Vector3D,
}


impl Particle {
    /// Create a new `Particle` from a `name`
    pub fn new<S: Into<String>>(name: S) -> Particle {
        Particle::with_position(name, Vector3D::zero())
    }

    /// Create a new `Particle` from a `name` and a `position`
    pub fn with_position<S: Into<String>>(name: S, position: Vector3D) -> Particle {
        let name = name.into();
        let mass = PeriodicTable::mass(&name).unwrap_or_else(||{
            warn_once!("Could not find the mass for the {} particle", name);
            return 0.0;
        });
        Particle {
            name: name,
            mass: mass,
            charge: 0.0,
            kind: ParticleKind::invalid(),
            position: position,
            velocity: Vector3D::zero()
        }
    }

    /// Get the particle name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the particle name to `name`
    pub fn set_name<'a, S>(&mut self, name: S) where S: Into<&'a str> {
        self.name = String::from(name.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Vector3D;

    #[test]
    fn mass_initialization() {
        let particle = Particle::new("O");
        assert_eq!(particle.mass, 15.999);
    }

    #[test]
    fn name() {
        let mut particle = Particle::new("");
        assert_eq!(particle.name(), "");

        assert_eq!(particle.mass, 0.0);
        assert_eq!(particle.charge, 0.0);
        assert_eq!(particle.kind, ParticleKind::invalid());
        assert_eq!(particle.position, Vector3D::new(0.0, 0.0, 0.0));
        assert_eq!(particle.velocity, Vector3D::new(0.0, 0.0, 0.0));

        particle.set_name("H");
        assert_eq!(particle.name(), "H");
    }

    #[test]
    fn with_position() {
        let particle = Particle::with_position("", Vector3D::new(1.0, 2.0, 3.0));
        assert_eq!(particle.name(), "");
        assert_eq!(particle.position, Vector3D::new(1.0, 2.0, 3.0));

        assert_eq!(particle.mass, 0.0);
        assert_eq!(particle.charge, 0.0);
        assert_eq!(particle.kind, ParticleKind::invalid());
        assert_eq!(particle.velocity, Vector3D::new(0.0, 0.0, 0.0));
    }
}
