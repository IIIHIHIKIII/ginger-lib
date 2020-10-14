use super::quadratic_extension::*;
use crate::fields::{fp6_3over2::*, Field, Fp2, Fp2Parameters};
use std::marker::PhantomData;
use std::ops::{AddAssign, SubAssign};

type Fp2Params<P> = <<P as Fp12Parameters>::Fp6Params as Fp6Parameters>::Fp2Params;

pub trait Fp12Parameters: 'static + Send + Sync + Copy {
    type Fp6Params: Fp6Parameters;

    /// This *must* equal (0, 1, 0);
    /// see [[DESD06, Section 6.1]](https://eprint.iacr.org/2006/471.pdf).
    const NONRESIDUE: Fp6<Self::Fp6Params>;

    /// Coefficients for the Frobenius automorphism.
    const FROBENIUS_COEFF_FP12_C1: &'static [Fp2<Fp2Params<Self>>];

    /// Multiply by quadratic nonresidue v.
    #[inline(always)]
    fn mul_fp6_by_nonresidue(fe: &Fp6<Self::Fp6Params>) -> Fp6<Self::Fp6Params> {
        // see [[DESD06, Section 6.1]](https://eprint.iacr.org/2006/471.pdf).
        let new_c0 = Self::Fp6Params::mul_fp2_by_nonresidue(&fe.c2);
        let new_c1 = fe.c0;
        let new_c2 = fe.c1;
        Fp6::new(new_c0, new_c1, new_c2)
    }
}

pub struct Fp12ParamsWrapper<P: Fp12Parameters>(PhantomData<P>);

impl<P: Fp12Parameters> QuadExtParameters for Fp12ParamsWrapper<P> {
    type BasePrimeField = <Fp2Params<P> as Fp2Parameters>::Fp;
    type BaseField = Fp6<P::Fp6Params>;
    type FrobCoeff = Fp2<Fp2Params<P>>;

    const DEGREE_OVER_BASE_PRIME_FIELD: usize = 12;

    const NONRESIDUE: Self::BaseField = P::NONRESIDUE;

    const FROBENIUS_COEFF_C1: &'static [Self::FrobCoeff] = P::FROBENIUS_COEFF_FP12_C1;

    #[inline(always)]
    fn mul_base_field_by_nonresidue(fe: &Self::BaseField) -> Self::BaseField {
        P::mul_fp6_by_nonresidue(fe)
    }

    fn mul_base_field_by_frob_coeff(fe: &mut Self::BaseField, power: usize) {
        fe.mul_assign_by_fp2(Self::FROBENIUS_COEFF_C1[power % Self::DEGREE_OVER_BASE_PRIME_FIELD]);
    }

    fn cyclotomic_square(fe: &QuadExtField<Self>) -> QuadExtField<Self> {
        // Faster Squaring in the Cyclotomic Subgroup of Sixth Degree Extensions
        // - Robert Granger and Michael Scott
        //
        if characteristic_square_mod_6_is_one(QuadExtField::<Self>::characteristic()) {
            let mut result = QuadExtField::<Self>::zero();
            let fp2_nr = <P::Fp6Params as Fp6Parameters>::mul_fp2_by_nonresidue;

            let mut z0 = fe.c0.c0;
            let mut z4 = fe.c0.c1;
            let mut z3 = fe.c0.c2;
            let mut z2 = fe.c1.c0;
            let mut z1 = fe.c1.c1;
            let mut z5 = fe.c1.c2;

            // t0 + t1*y = (z0 + z1*y)^2 = a^2
            let mut tmp = z0 * &z1;
            let t0 = (z0 + &z1) * &(z0 + &fp2_nr(&z1)) - &tmp - &fp2_nr(&tmp);
            let t1 = tmp.double();

            // t2 + t3*y = (z2 + z3*y)^2 = b^2
            tmp = z2 * &z3;
            let t2 = (z2 + &z3) * &(z2 + &fp2_nr(&z3)) - &tmp - &fp2_nr(&tmp);
            let t3 = tmp.double();

            // t4 + t5*y = (z4 + z5*y)^2 = c^2
            tmp = z4 * &z5;
            let t4 = (z4 + &z5) * &(z4 + &fp2_nr(&z5)) - &tmp - &fp2_nr(&tmp);
            let t5 = tmp.double();

            // for A

            // z0 = 3 * t0 - 2 * z0
            z0 = t0 - &z0;
            z0 = z0 + &z0;
            result.c0.c0 = z0 + &t0;

            // z1 = 3 * t1 + 2 * z1
            z1 = t1 + &z1;
            z1 = z1 + &z1;
            result.c1.c1 = z1 + &t1;

            // for B

            // z2 = 3 * (xi * t5) + 2 * z2
            tmp = fp2_nr(&t5);
            z2 = tmp + &z2;
            z2 = z2 + &z2;
            result.c1.c0 = z2 + &tmp;

            // z3 = 3 * t4 - 2 * z3
            z3 = t4 - &z3;
            z3 = z3 + &z3;
            result.c0.c2 = z3 + &t4;

            // for C

            // z4 = 3 * t2 - 2 * z4
            z4 = t2 - &z4;
            z4 = z4 + &z4;
            result.c0.c1 = z4 + &t2;

            // z5 = 3 * t3 + 2 * z5
            z5 = t3 + &z5;
            z5 = z5 + &z5;
            result.c1.c2 = z5 + &t3;

            result
        } else {
            fe.square()
        }
    }
}

pub type Fp12<P> = QuadExtField<Fp12ParamsWrapper<P>>;

impl<P: Fp12Parameters> Fp12<P> {
    pub fn mul_by_fp(
        &mut self,
        element: &<<P::Fp6Params as Fp6Parameters>::Fp2Params as Fp2Parameters>::Fp,
    ) {
        self.c0.mul_by_fp(element);
        self.c1.mul_by_fp(element);
    }

    pub fn mul_by_034(
        &mut self,
        c0: &Fp2<Fp2Params<P>>,
        c3: &Fp2<Fp2Params<P>>,
        c4: &Fp2<Fp2Params<P>>,
    ) {
        let a0 = self.c0.c0 * c0;
        let a1 = self.c0.c1 * c0;
        let a2 = self.c0.c2 * c0;
        let a = Fp6::new(a0, a1, a2);
        let mut b = self.c1;
        b.mul_by_01(&c3, &c4);

        let c0 = *c0 + c3;
        let c1 = c4;
        let mut e = self.c0 + &self.c1;
        e.mul_by_01(&c0, &c1);
        self.c1 = e - &(a + &b);
        self.c0 = a + &P::mul_fp6_by_nonresidue(&b);
    }

    pub fn mul_by_014(
        &mut self,
        c0: &Fp2<Fp2Params<P>>,
        c1: &Fp2<Fp2Params<P>>,
        c4: &Fp2<Fp2Params<P>>,
    ) {
        let mut aa = self.c0;
        aa.mul_by_01(c0, c1);
        let mut bb = self.c1;
        bb.mul_by_1(c4);
        let mut o = *c1;
        o.add_assign(c4);
        self.c1.add_assign(&self.c0);
        self.c1.mul_by_01(c0, &o);
        self.c1.sub_assign(&aa);
        self.c1.sub_assign(&bb);
        self.c0 = bb;
        self.c0 = P::mul_fp6_by_nonresidue(&self.c0);
        self.c0.add_assign(&aa);
    }
}

// TODO: make `const fn` in 1.46.
pub(crate) fn characteristic_square_mod_6_is_one(characteristic: &[u64]) -> bool {
    let mut characteristic_mod_3 = 0i64;
    for limb in characteristic {
        for i in 0..64i64 {
            let bit = ((limb >> i) & 1) as i64;
            let b = if i % 2 == 0 { bit } else { -bit };
            characteristic_mod_3 = (characteristic_mod_3 + b) % 3;
        }
    }
    let characteristic_mod_2 = characteristic[0] % 2;
    (characteristic_mod_3 != 0) && (characteristic_mod_2 == 1)
}