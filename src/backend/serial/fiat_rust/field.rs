// -*- mode: rust; coding: utf-8; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2018 Isis Lovecruft, Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - Isis Agora Lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

//! Field arithmetic modulo \\(p = 2\^{255} - 19\\), using \\(64\\)-bit
//! limbs with \\(128\\)-bit products.

use core::fmt::Debug;
use core::ops::Neg;
use core::ops::{Add, AddAssign};
use core::ops::{Mul, MulAssign};
use core::ops::{Sub, SubAssign};

use subtle::Choice;
use subtle::ConditionallySelectable;

use curve25519_fiat::curve25519_64::*;

/// A `FieldElement51` represents an element of the field
/// \\( \mathbb Z / (2\^{255} - 19)\\).
///
/// In the 64-bit implementation, a `FieldElement` is represented in
/// radix \\(2\^{51}\\) as five `u64`s; the coefficients are allowed to
/// grow up to \\(2\^{54}\\) between reductions modulo \\(p\\).
///
/// # Note
///
/// The `curve25519_dalek::field` module provides a type alias
/// `curve25519_dalek::field::FieldElement` to either `FieldElement51`
/// or `FieldElement2625`.
///
/// The backend-specific type `FieldElement51` should not be used
/// outside of the `curve25519_dalek::field` module.
#[derive(Copy, Clone)]
pub struct FieldElement51(pub(crate) [u64; 5]);

impl Debug for FieldElement51 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "FieldElement51({:?})", &self.0[..])
    }
}

impl<'b> AddAssign<&'b FieldElement51> for FieldElement51 {
    fn add_assign(&mut self, _rhs: &'b FieldElement51) {
        let input = self.0;
        fiat_25519_add(&mut self.0, &input, &_rhs.0);
        let input = self.0;
        fiat_25519_carry(&mut self.0, &input);
    }
}

impl<'a, 'b> Add<&'b FieldElement51> for &'a FieldElement51 {
    type Output = FieldElement51;
    fn add(self, _rhs: &'b FieldElement51) -> FieldElement51 {
        let mut output = *self;
        fiat_25519_add(&mut output.0, &self.0, &_rhs.0);
        let input = output.0;
        fiat_25519_carry(&mut output.0, &input);
        output
    }
}

impl<'b> SubAssign<&'b FieldElement51> for FieldElement51 {
    fn sub_assign(&mut self, _rhs: &'b FieldElement51) {
        let input = self.0;
        fiat_25519_sub(&mut self.0, &input, &_rhs.0);
        let input = self.0;
        fiat_25519_carry(&mut self.0, &input);
    }
}

impl<'a, 'b> Sub<&'b FieldElement51> for &'a FieldElement51 {
    type Output = FieldElement51;
    fn sub(self, _rhs: &'b FieldElement51) -> FieldElement51 {
        let mut output = *self;
        fiat_25519_sub(&mut output.0, &self.0, &_rhs.0);
        let input = output.0;
        fiat_25519_carry(&mut output.0, &input);
        output
    }
}

impl<'b> MulAssign<&'b FieldElement51> for FieldElement51 {
    fn mul_assign(&mut self, _rhs: &'b FieldElement51) {
        let input = self.0;
        fiat_25519_carry_mul(&mut self.0, &input, &_rhs.0);
    }
}

impl<'a, 'b> Mul<&'b FieldElement51> for &'a FieldElement51 {
    type Output = FieldElement51;
    fn mul(self, _rhs: &'b FieldElement51) -> FieldElement51 {
        let mut output = *self;
        fiat_25519_carry_mul(&mut output.0, &self.0, &_rhs.0);
        output
    }
}

impl<'a> Neg for &'a FieldElement51 {
    type Output = FieldElement51;
    fn neg(self) -> FieldElement51 {
        let mut output = *self;
        fiat_25519_opp(&mut output.0, &self.0);
        let input = output.0;
        fiat_25519_carry(&mut output.0, &input);
        output
    }
}

impl ConditionallySelectable for FieldElement51 {
    fn conditional_select(
        a: &FieldElement51,
        b: &FieldElement51,
        choice: Choice,
    ) -> FieldElement51 {
        let mut output = [0u64; 5];;
        fiat_25519_selectznz(&mut output, choice.unwrap_u8() as fiat_25519_u1, &a.0, &b.0);
        FieldElement51(output)
    }

    fn conditional_swap(a: &mut FieldElement51, b: &mut FieldElement51, choice: Choice) {
        u64::conditional_swap(&mut a.0[0], &mut b.0[0], choice);
        u64::conditional_swap(&mut a.0[1], &mut b.0[1], choice);
        u64::conditional_swap(&mut a.0[2], &mut b.0[2], choice);
        u64::conditional_swap(&mut a.0[3], &mut b.0[3], choice);
        u64::conditional_swap(&mut a.0[4], &mut b.0[4], choice);
    }

    fn conditional_assign(&mut self, _rhs: &FieldElement51, choice: Choice) {
        self.0[0].conditional_assign(&_rhs.0[0], choice);
        self.0[1].conditional_assign(&_rhs.0[1], choice);
        self.0[2].conditional_assign(&_rhs.0[2], choice);
        self.0[3].conditional_assign(&_rhs.0[3], choice);
        self.0[4].conditional_assign(&_rhs.0[4], choice);
    }
}

impl FieldElement51 {
    /// Construct zero.
    pub fn zero() -> FieldElement51 {
        FieldElement51([0, 0, 0, 0, 0])
    }

    /// Construct one.
    pub fn one() -> FieldElement51 {
        FieldElement51([1, 0, 0, 0, 0])
    }

    /// Construct -1.
    pub fn minus_one() -> FieldElement51 {
        FieldElement51([
            2251799813685228,
            2251799813685247,
            2251799813685247,
            2251799813685247,
            2251799813685247,
        ])
    }

    /// Given 64-bit input limbs, reduce to enforce the bound 2^(51 + epsilon).
    #[inline(always)]
    #[allow(dead_code)] // Need this to not complain about reduce not being used
    fn reduce(mut limbs: [u64; 5]) -> FieldElement51 {
        let input = limbs;
        fiat_25519_carry(&mut limbs, &input);
        FieldElement51(limbs)
    }

    /// Load a `FieldElement51` from the low 255 bits of a 256-bit
    /// input.
    ///
    /// # Warning
    ///
    /// This function does not check that the input used the canonical
    /// representative.  It masks the high bit, but it will happily
    /// decode 2^255 - 18 to 1.  Applications that require a canonical
    /// encoding of every field element should decode, re-encode to
    /// the canonical encoding, and check that the input was
    /// canonical.
    ///
    pub fn from_bytes(bytes: &[u8; 32]) -> FieldElement51 {
        let mut temp = [0u8; 32];
        temp.copy_from_slice(bytes);
        temp[31] &= 127u8;
        let mut output = [0u64; 5];
        fiat_25519_from_bytes(&mut output, &temp);
        FieldElement51(output)
    }

    /// Serialize this `FieldElement51` to a 32-byte array.  The
    /// encoding is canonical.
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        fiat_25519_to_bytes(&mut bytes, &self.0);
        return bytes;
    }

    /// Given `k > 0`, return `self^(2^k)`.
    pub fn pow2k(&self, mut k: u32) -> FieldElement51 {
        let mut output = *self;
        loop {
            let input = output.0;
            fiat_25519_carry_square(&mut output.0, &input);
            k -= 1;
            if k == 0 {
                return output;
            }
        }
    }

    /// Returns the square of this field element.
    pub fn square(&self) -> FieldElement51 {
        let mut output = *self;
        fiat_25519_carry_square(&mut output.0, &self.0);
        output
    }

    /// Returns 2 times the square of this field element.
    pub fn square2(&self) -> FieldElement51 {
        let mut output = *self;
        let mut temp = *self;
        // Void vs return type, measure cost of copying self
        fiat_25519_carry_square(&mut temp.0, &self.0);
        fiat_25519_add(&mut output.0, &temp.0, &temp.0);
        let input = output.0;
        fiat_25519_carry(&mut output.0, &input);
        output
    }
}
