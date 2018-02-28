// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// https://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Basic floating-point number distributions

use core::mem;
use Rng;
use distributions::{Distribution, Uniform};


/// A distribution to sample floating point numbers uniformly in the open
/// interval `(0, 1)` (not including either endpoint).
///
/// See also: [`Closed01`] for the closed `[0, 1]`; [`Uniform`] for the
/// half-open `[0, 1)`.
///
/// # Example
/// ```rust
/// use rand::{weak_rng, Rng};
/// use rand::distributions::Open01;
///
/// let val: f32 = weak_rng().sample(Open01);
/// println!("f32 from (0,1): {}", val);
/// ```
///
/// [`Uniform`]: struct.Uniform.html
/// [`Closed01`]: struct.Closed01.html
#[derive(Clone, Copy, Debug)]
pub struct Open01;

/// A distribution to sample floating point numbers uniformly in the closed
/// interval `[0, 1]` (including both endpoints).
///
/// See also: [`Open01`] for the open `(0, 1)`; [`Uniform`] for the half-open
/// `[0, 1)`.
///
/// # Example
/// ```rust
/// use rand::{weak_rng, Rng};
/// use rand::distributions::Closed01;
///
/// let val: f32 = weak_rng().sample(Closed01);
/// println!("f32 from [0,1]: {}", val);
/// ```
///
/// [`Uniform`]: struct.Uniform.html
/// [`Open01`]: struct.Open01.html
#[derive(Clone, Copy, Debug)]
pub struct Closed01;


// Return the next random f32 selected from the half-open
// interval `[0, 1)`.
//
// This uses a technique described by Saito and Matsumoto at
// MCQMC'08. Given that the IEEE floating point numbers are
// uniformly distributed over [1,2), we generate a number in
// this range and then offset it onto the range [0,1). Our
// choice of bits (masking v. shifting) is arbitrary and
// should be immaterial for high quality generators. For low
// quality generators (ex. LCG), prefer bitshifting due to
// correlation between sequential low order bits.
//
// See:
// A PRNG specialized in double precision floating point numbers using
// an affine transition
//
// * <http://www.math.sci.hiroshima-u.ac.jp/~m-mat/MT/ARTICLES/dSFMT.pdf>
// * <http://www.math.sci.hiroshima-u.ac.jp/~m-mat/MT/SFMT/dSFMT-slide-e.pdf>
impl Distribution<f32> for Uniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        const UPPER_MASK: u32 = 0x3F800000;
        const LOWER_MASK: u32 = 0x7FFFFF;
        let tmp = UPPER_MASK | (rng.next_u32() & LOWER_MASK);
        let result: f32 = unsafe { mem::transmute(tmp) };
        result - 1.0
    }
}
impl Distribution<f64> for Uniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        const UPPER_MASK: u64 = 0x3FF0000000000000;
        const LOWER_MASK: u64 = 0xFFFFFFFFFFFFF;
        let tmp = UPPER_MASK | (rng.next_u64() & LOWER_MASK);
        let result: f64 = unsafe { mem::transmute(tmp) };
        result - 1.0
    }
}

macro_rules! float_impls {
    ($mod_name:ident, $ty:ty, $mantissa_bits:expr) => {
        mod $mod_name {
            use Rng;
            use distributions::{Distribution};
            use super::{Open01, Closed01};

            const SCALE: $ty = (1u64 << $mantissa_bits) as $ty;

            impl Distribution<$ty> for Open01 {
                #[inline]
                fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $ty {
                    // add 0.5 * epsilon, so that smallest number is
                    // greater than 0, and largest number is still
                    // less than 1, specifically 1 - 0.5 * epsilon.
                    let x: $ty = rng.gen();
                    x + 0.5 / SCALE
                }
            }
            impl Distribution<$ty> for Closed01 {
                #[inline]
                fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $ty {
                    // rescale so that 1.0 - epsilon becomes 1.0
                    // precisely.
                    let x: $ty = rng.gen();
                    x * SCALE / (SCALE - 1.0)
                }
            }
        }
    }
}
float_impls! { f64_rand_impls, f64, 52 }
float_impls! { f32_rand_impls, f32, 23 }


#[cfg(test)]
mod tests {
    use Rng;
    use mock::StepRng;
    use distributions::{Open01, Closed01};

    const EPSILON32: f32 = ::core::f32::EPSILON;
    const EPSILON64: f64 = ::core::f64::EPSILON;

    #[test]
    fn floating_point_edge_cases() {
        let mut zeros = StepRng::new(0, 0);
        assert_eq!(zeros.gen::<f32>(), 0.0);
        assert_eq!(zeros.gen::<f64>(), 0.0);
        
        let mut one = StepRng::new(1, 0);
        assert_eq!(one.gen::<f32>(), EPSILON32);
        assert_eq!(one.gen::<f64>(), EPSILON64);
        
        let mut max = StepRng::new(!0, 0);
        assert_eq!(max.gen::<f32>(), 1.0 - EPSILON32);
        assert_eq!(max.gen::<f64>(), 1.0 - EPSILON64);
    }

    #[test]
    fn fp_closed_edge_cases() {
        let mut zeros = StepRng::new(0, 0);
        assert_eq!(zeros.sample::<f32, _>(Closed01), 0.0);
        assert_eq!(zeros.sample::<f64, _>(Closed01), 0.0);
        
        let mut one = StepRng::new(1, 0);
        let one32 = one.sample::<f32, _>(Closed01);
        let one64 = one.sample::<f64, _>(Closed01);
        assert!(EPSILON32 < one32 && one32 < EPSILON32 * 1.01);
        assert!(EPSILON64 < one64 && one64 < EPSILON64 * 1.01);
        
        let mut max = StepRng::new(!0, 0);
        assert_eq!(max.sample::<f32, _>(Closed01), 1.0);
        assert_eq!(max.sample::<f64, _>(Closed01), 1.0);
    }

    #[test]
    fn fp_open_edge_cases() {
        let mut zeros = StepRng::new(0, 0);
        assert_eq!(zeros.sample::<f32, _>(Open01), 0.0 + EPSILON32 / 2.0);
        assert_eq!(zeros.sample::<f64, _>(Open01), 0.0 + EPSILON64 / 2.0);
        
        let mut one = StepRng::new(1, 0);
        let one32 = one.sample::<f32, _>(Open01);
        let one64 = one.sample::<f64, _>(Open01);
        assert!(EPSILON32 < one32 && one32 < EPSILON32 * 2.0);
        assert!(EPSILON64 < one64 && one64 < EPSILON64 * 2.0);
        
        let mut max = StepRng::new(!0, 0);
        assert_eq!(max.sample::<f32, _>(Open01), 1.0 - EPSILON32 / 2.0);
        assert_eq!(max.sample::<f64, _>(Open01), 1.0 - EPSILON64 / 2.0);
    }

    #[test]
    fn rand_open() {
        // this is unlikely to catch an incorrect implementation that
        // generates exactly 0 or 1, but it keeps it sane.
        let mut rng = ::test::rng(510);
        for _ in 0..1_000 {
            // strict inequalities
            let f: f64 = rng.sample(Open01);
            assert!(0.0 < f && f < 1.0);

            let f: f32 = rng.sample(Open01);
            assert!(0.0 < f && f < 1.0);
        }
    }

    #[test]
    fn rand_closed() {
        let mut rng = ::test::rng(511);
        for _ in 0..1_000 {
            // strict inequalities
            let f: f64 = rng.sample(Closed01);
            assert!(0.0 <= f && f <= 1.0);

            let f: f32 = rng.sample(Closed01);
            assert!(0.0 <= f && f <= 1.0);
        }
    }
}
