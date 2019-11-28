use algebra::{Field, AffineCurve};
use algebra::curves::models::mnt4::{g1::G1Prepared, g2::{G2PreparedCoefficients, G2Prepared}};

use crate::{fields::{
    FieldGadget, fp::FpGadget, fp2::Fp2Gadget,
}, groups::curves::short_weierstrass::AffineGadget,
    bits::uint8::UInt8, Assignment,
    alloc::{AllocGadget, HardCodedGadget},
    ToBytesGadget,
};

use r1cs_core::{ConstraintSystem, SynthesisError};
use algebra::curves::models::mnt4::MNT4Parameters;
use std::fmt::Debug;
use std::ops::{Mul, Sub};
use crate::groups::GroupGadget;
use std::borrow::Borrow;

pub mod mnt4753;

pub type G1Gadget<P> = AffineGadget<<P as MNT4Parameters>::G1Parameters, <P as MNT4Parameters>::Fp, FpG<P>>;
pub type G2Gadget<P> = AffineGadget<<P as MNT4Parameters>::G2Parameters, <P as MNT4Parameters>::Fp, Fp2G<P>>;

type FpG<P> = FpGadget<<P as MNT4Parameters>::Fp>;
type Fp2G<P> = Fp2Gadget<<P as MNT4Parameters>::Fp2Params, <P as MNT4Parameters>::Fp>;

#[derive(Derivative)]
#[derivative(
Clone(bound = "FpGadget<P::Fp>: Clone"),
Clone(bound = "Fp2Gadget<P::Fp2Params, P::Fp>: Clone"),
Debug(bound = "FpGadget<P::Fp>: Debug"),
Debug(bound = "Fp2Gadget<P::Fp2Params, P::Fp>: Debug"),
)]
pub struct G1PreparedGadget<P: MNT4Parameters> {
    pub p:                   G1Gadget<P>,
    pub p_y_twist_squared:   Fp2G<P>,
}

impl<P: MNT4Parameters> G1PreparedGadget<P> {
    pub fn from_affine<CS: ConstraintSystem<P::Fp>>(
        mut cs: CS,
        value: &G1Gadget<P>,
    ) -> Result<Self, SynthesisError> {

        let p = G1Gadget::<P>::alloc(cs.ns(|| "value into affine"),
                                        || value.clone().get_value().ok_or(SynthesisError::AssignmentMissing))?;
        let twist_squared = P::TWIST.square();
        let c0 = p.y.mul_by_constant(cs.ns(||"p.y * twist_squared.c0"), &twist_squared.c0)?;
        let c1 = p.y.mul_by_constant(cs.ns(||"p.y * twist_squared.c1"), &twist_squared.c1)?;
        let p_y_twist_squared = Fp2G::<P>::new(c0, c1);
        Ok(G1PreparedGadget{p, p_y_twist_squared})
    }
}

impl<P: MNT4Parameters> ToBytesGadget<P::Fp> for G1PreparedGadget<P> {
    #[inline]
    fn to_bytes<CS: ConstraintSystem<P::Fp>>(&self, mut cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        let mut p = self.p.to_bytes(&mut cs.ns(|| "p to bytes"))?;
        p.extend_from_slice(&self.p_y_twist_squared.to_bytes(&mut cs.ns(|| "p_y_twist_squared to bytes"))?);
        Ok(p)
    }

    fn to_bytes_strict<CS: ConstraintSystem<P::Fp>>(&self, mut cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        self.to_bytes(cs.ns(|| "to_bytes_g1_prepared"))
    }
}

impl<P: MNT4Parameters> HardCodedGadget<G1Prepared<P>, P::Fp> for G1PreparedGadget<P> {
    #[inline]
    fn alloc_hardcoded<F, T, CS: ConstraintSystem<P::Fp>>(mut cs: CS, value_gen: F) -> Result<Self, SynthesisError> where
        F: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<G1Prepared<P>>
    {
        value_gen().and_then(|g1p| {
            let G1Prepared {
                p,
                py_twist_squared,
            } = g1p.borrow().clone();

            let p = G1Gadget::<P>::alloc_hardcoded(cs.ns(|| "hardcode p"), || Ok(p.into_projective()))?;
            let p_y_twist_squared = {
                let mut t = Fp2G::<P>::zero(cs.ns(|| "alloc zero for p_y_twist_squared"))?;
                t.add_constant_in_place(cs.ns(|| "hardcode p_y_twist_squared"), &py_twist_squared)?;
                t
            };

            Ok(Self {
                p,
                p_y_twist_squared,
            })
        })
    }
}


#[derive(Derivative)]
#[derivative(
Clone(bound = "Fp2Gadget<P::Fp2Params, P::Fp>: Clone"),
Debug(bound = "Fp2Gadget<P::Fp2Params, P::Fp>: Debug")
)]
pub struct G2CoefficientsGadget<P: MNT4Parameters> {
    pub(crate) r_y:            Fp2G<P>,
    pub(crate) gamma:          Fp2G<P>,
    pub(crate) gamma_x:        Fp2G<P>,
}

impl<P: MNT4Parameters> ToBytesGadget<P::Fp> for G2CoefficientsGadget<P> {
    #[inline]
    fn to_bytes<CS: ConstraintSystem<P::Fp>>(&self, mut cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        let mut x = self.r_y.to_bytes(&mut cs.ns(|| "r_y to bytes"))?;
        x.extend_from_slice(&self.gamma.to_bytes(&mut cs.ns(|| "gamma to bytes"))?);
        x.extend_from_slice(&self.gamma_x.to_bytes(&mut cs.ns(|| "gamma_x to bytes"))?);
        Ok(x)
    }

    fn to_bytes_strict<CS: ConstraintSystem<P::Fp>>(&self, mut cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        self.to_bytes(cs.ns(|| "to_bytes AteDoubleCoefficients"))
    }
}

impl<P: MNT4Parameters> HardCodedGadget<G2PreparedCoefficients<P>, P::Fp> for G2CoefficientsGadget<P> {
    #[inline]
    fn alloc_hardcoded<F, T, CS: ConstraintSystem<P::Fp>>(mut cs: CS, value_gen: F) -> Result<Self, SynthesisError> where
        F: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<G2PreparedCoefficients<P>> {
        value_gen().and_then(|g2pc| {
            let G2PreparedCoefficients {
                r_y,
                gamma,
                gamma_x,
            } = g2pc.borrow().clone();


            let r_y = {
                let mut t = Fp2G::<P>::zero(cs.ns(|| "alloc zero for r_y"))?;
                t.add_constant_in_place(cs.ns(|| "hardcode r_y"), &r_y)?;
                t
            };

            let gamma = {
                let mut t = Fp2G::<P>::zero(cs.ns(|| "alloc zero for gamma"))?;
                t.add_constant_in_place(cs.ns(|| "hardcode gamma"), &gamma)?;
                t
            };

            let gamma_x = {
                let mut t = Fp2G::<P>::zero(cs.ns(|| "alloc zero for gamma_x"))?;
                t.add_constant_in_place(cs.ns(|| "hardcode gamma_x"), &gamma_x)?;
                t
            };

            Ok(Self {
                r_y,
                gamma,
                gamma_x,
            })
        })
    }
}




#[derive(Derivative)]
#[derivative(
Clone(bound = "Fp2Gadget<P::Fp2Params, P::Fp>: Clone"),
Debug(bound = "Fp2Gadget<P::Fp2Params, P::Fp>: Debug")
)]
pub struct G2PreparedGadget<P: MNT4Parameters>{
    pub q:      G2Gadget<P>,
    pub coeffs: Vec<G2CoefficientsGadget<P>>
}

impl<P: MNT4Parameters>G2PreparedGadget<P> {
    pub fn from_affine<CS: ConstraintSystem<P::Fp>>(
        mut cs: CS,
        value: &G2Gadget<P>,
    ) -> Result<Self, SynthesisError> {

        //Workaround to convert into affine without using unwrap
        let mut s = G2Gadget::<P>::alloc(cs.ns(|| "value into affine"),
                                || value.clone().get_value().ok_or(SynthesisError::AssignmentMissing))?;

        let mut g2p = G2PreparedGadget{
            q: s.clone(),
            coeffs: vec![]
        };

        for (i, &n) in P::WNAF.iter().rev().enumerate(){

            let mut cs = cs.ns(|| format!("Iteration {}", i));

            let (s2, c) = Self::doubling_step_for_flipped_miller_loop(cs.ns(|| "double"), &s.clone())?;
            g2p.coeffs.push(c);
            s = s2;
            if n != 0 {
                let (s2, c) = Self::mixed_addition_step_for_flipped_miller_loop(cs.ns(|| "add"), &value.x, &value.y, &s.clone(), n)?;
                g2p.coeffs.push(c);
                s = s2;
            }
        }
        Ok(g2p)
    }

    fn doubling_step_for_flipped_miller_loop<CS: ConstraintSystem<P::Fp>>(
        mut cs: CS,
        s: &G2Gadget<P>,
    ) -> Result<(G2Gadget<P>, G2CoefficientsGadget<P>), SynthesisError>
    {
        //Compute gamma
        let s_x_squared = s.x.square(cs.ns(||"s_x^2"))?;
        let three_sx_squared_plus_a = s_x_squared
            .double(cs.ns(|| "2s_x^2"))?
            .add(cs.ns(|| "3s_x^2"), &s_x_squared)?
            .add_constant(cs.ns(|| "3s_x^2 + a"), &P::TWIST_COEFF_A)?;

        let two_sy = s.y.double(cs.ns(||"2s_y"))?;

        let gamma = Fp2G::<P>::alloc(cs.ns(|| "gamma"), || {
            Ok(three_sx_squared_plus_a.get_value().get()?.mul(&two_sy.get_value().get()?.inverse().get()?))
        })?;

        //Check gamma (gamma*2s_y = sx^2 + 3a)
        gamma.mul_equals(cs.ns(|| "Check gamma"), &two_sy, &three_sx_squared_plus_a)?;

        //Compute and check gamma_x
        let gamma_x = gamma.mul(cs.ns(|| "Compute gamma_x"), &s.x)?;

        //Compute new_sx
        let two_sx = s.x.double(cs.ns(|| "2s_x"))?;
        let new_sx = Fp2G::<P>::alloc(cs.ns(|| "Compute new_sx"), || {
            let gamma_value = gamma.get_value().get()?;
            Ok(gamma_value.mul(&gamma_value).sub(&two_sx.get_value().get()?))
        })?;

        //Check new_sx: new_sx+2s_x = gamma*gamma
        let new_s_x_plus_2_s_x = new_sx.add(cs.ns(|| "new_s_x + 2_s_x"), &two_sx)?;
        gamma.mul_equals(cs.ns(|| "new sx calculation is correct"), &gamma, &new_s_x_plus_2_s_x)?;

        //Compute new_sy
        let s_x_minus_new_s_x = s.x
            .sub(cs.ns(|| "s_x - new_s_x"), &new_sx)?;

        let new_sy = Fp2G::<P>::alloc(cs.ns(|| "Compute new_sy"), || {
           Ok(s_x_minus_new_s_x
               .get_value().get()?
               .mul(&gamma.get_value().get()?)
               .sub(&s.y.get_value().get()?))
        })?;

        let new_sy_plus_s_y = new_sy.add(cs.ns(|| "new_sy + sy"), &s.y)?;

        //Check new_sy: new_sy + sy = gamma*(sx - new_sx)
        gamma.mul_equals(cs.ns(|| "Check new_sy"), &s_x_minus_new_s_x, &new_sy_plus_s_y)?;

        let c = G2CoefficientsGadget{r_y: s.y.clone(), gamma, gamma_x};
        let s2 = G2Gadget::<P>::new(new_sx, new_sy);

        Ok((s2, c))
    }

    fn mixed_addition_step_for_flipped_miller_loop<CS: ConstraintSystem<P::Fp>>(
        mut cs: CS,
        x: &Fp2G<P>,
        y: &Fp2G<P>,
        s: &G2Gadget<P>,
        naf_i: i32,
    ) -> Result<(G2Gadget<P>, G2CoefficientsGadget<P>), SynthesisError>
    {
        //Compute gamma
        let sx_minus_x = s.x
            .sub(cs.ns(|| "s_x - x"), &x)?;

        let sy_plus_y = s.y.add(cs.ns(||"(s_y + y)"), &y)?;
        let sy_minus_y = s.y.sub(cs.ns(|| "(s_y - y)"), &y)?;
        let numerator = if naf_i > 0 {sy_minus_y} else {sy_plus_y};

        let gamma = Fp2G::<P>::alloc(cs.ns(|| "Compute gamma"), || {
            let sx_minus_x_inv = sx_minus_x.get_value().get()?.inverse().get()?;
            Ok(numerator.get_value().get()?.mul(&sx_minus_x_inv))
        })?;

        //Check gamma
        gamma.mul_equals(cs.ns(||"Check gamma"), &sx_minus_x, &numerator)?;

        //Compute gamma_x
        let gamma_x = gamma.mul(cs.ns(|| "Compute gamma_x"), &x)?;

        //Compute new_sx
        let new_sx = Fp2G::<P>::alloc(cs.ns(|| "Compute new_sx"), || {
            let gamma_value = gamma.get_value().get()?;
            Ok(gamma_value.mul(&gamma_value).sub(&s.x.get_value().get()?).sub(&x.get_value().get()?))
        })?;

        //Check new_sx
        let x_plus_s_x_plus_new_sx = x
            .add(cs.ns(|| "x + s_x"), &s.x)?
            .add(cs.ns(|| "x + s_x + new_s_x"), &new_sx)?;

        gamma.mul_equals(cs.ns(|| "Check new_sx"), &gamma, &x_plus_s_x_plus_new_sx)?;

        //Compute new_sy
        let s_x_minus_new_s_x = s.x
            .sub(cs.ns(|| "s_x - new_s_x"), &new_sx)?;

        let new_sy = Fp2G::<P>::alloc(cs.ns(|| "Compute new_sy"), || {
            Ok(s_x_minus_new_s_x
                .get_value().get()?
                .mul(&gamma.get_value().get()?)
                .sub(&s.y.get_value().get()?))
        })?;

        let new_sy_plus_s_y = new_sy.add(cs.ns(|| "new_sy + sy"), &s.y)?;

        //Check new_sy: new_sy + sy = gamma*(sx - new_sx)
        gamma.mul_equals(cs.ns(|| "Check new_sy"), &s_x_minus_new_s_x, &new_sy_plus_s_y)?;

        let c = G2CoefficientsGadget{r_y: s.y.clone(), gamma, gamma_x};
        let s2 = G2Gadget::<P>::new(new_sx, new_sy);

        Ok((s2, c))
    }
}

impl<P: MNT4Parameters> ToBytesGadget<P::Fp> for G2PreparedGadget<P>
{
    #[inline]
    fn to_bytes<CS: ConstraintSystem<P::Fp>>(&self, mut cs: CS) -> Result<Vec<UInt8>, SynthesisError>
    {
        let mut x = self.q.to_bytes(&mut cs.ns(|| "q to bytes"))?;

        for (i, c) in self.coeffs.iter().enumerate() {
            x.extend_from_slice(&c.to_bytes(&mut cs.ns(|| format!("coefficients {} to bytes", i)))?);
        }

        Ok(x)
    }

    fn to_bytes_strict<CS: ConstraintSystem<P::Fp>>(&self, mut cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        self.to_bytes(cs.ns(|| "to_bytes_g2_prepared"))
    }
}

impl<P: MNT4Parameters> HardCodedGadget<G2Prepared<P>, P::Fp> for G2PreparedGadget<P> {
    #[inline]
    fn alloc_hardcoded<F, T, CS: ConstraintSystem<P::Fp>>(mut cs: CS, value_gen: F) -> Result<Self, SynthesisError>
        where
        F: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<G2Prepared<P>>
    {
        value_gen().and_then(|g2p| {
            let G2Prepared {
                q,
                coeffs,
            } = g2p.borrow().clone();

            let q = G2Gadget::<P>::alloc_hardcoded(cs.ns(|| "hardcode q"), || Ok(q.into_projective()))?;
            let coeffs = coeffs
                .into_iter()
                .enumerate()
                .map(|(i, query_i)| {
                    G2CoefficientsGadget::<P>::alloc_hardcoded(cs.ns(|| format!("coeff_{}", i)), || {
                        Ok(query_i)
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Result<_, _>>()?;
            Ok(Self {
                q,
                coeffs,
            })
        })
    }
}