//! A bare-metal (thumbv7em-none-eabihf) monitoring sketch using only the
//! `no_std`, no-alloc building blocks. It builds for the host as an `rlib` and
//! cross-compiles cleanly to a Cortex-M target with `cargo build
//! --target thumbv7em-none-eabihf -p example-baremetal-thumbv7`.
#![no_std]

use tpt_zero_formal::bounded::BoundedInt;
use tpt_zero_formal::contract::{ensures, requires};
use tpt_zero_formal::invariant::{check_invariant, Invariant};

/// A temperature sensor whose reading is bounded to its calibrated envelope.
#[derive(Clone, Copy)]
pub struct Sensor {
    pub temp_c: BoundedInt<-40, 85>,
}

impl Invariant for Sensor {
    fn check(&self) -> bool {
        let v = self.temp_c.value();
        (-40i64..=85).contains(&v)
    }
}

/// Decodes a raw ADC reading into a validated sensor reading.
pub fn read_sensor(raw: i64) -> Sensor {
    requires!((-100i64..=200).contains(&raw), "raw adc within plausible range");
    let s = Sensor {
        temp_c: BoundedInt::new_clamped(raw),
    };
    ensures!(s.check(), "sensor reading within envelope");
    s
}

/// One monitoring step; returns the clamped, invariant-checked temperature.
pub fn monitor_step(raw: i64) -> BoundedInt<-40, 85> {
    let s = read_sensor(raw);
    let s = check_invariant!(s);
    s.temp_c
}
