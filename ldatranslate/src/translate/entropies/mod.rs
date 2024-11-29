mod errors;
pub use errors::*;

use evalexpr::export::evalexpr_num::Float;
use ndarray::{Array, ArrayBase, Data, Dimension, Zip};
use ndarray_stats::EntropyExt;
use ndarray_stats::errors::{EmptyInput, MultiInputError, ShapeMismatch};
use nom::Parser;
use num::{cast};

pub trait EntropyExt2<A, S, D>
where
    S: Data<Elem = A>,
    D: Dimension,
{
    /// https://en.wikipedia.org/wiki/R%C3%A9nyi_entropy#Definition
    fn renyi_entropy<A2>(&self, alpha: A2) -> Result<A, EntropyWithAlphaError<A, A2>> where A: Float, A2: Float;

    /// https://en.wikipedia.org/wiki/R%C3%A9nyi_entropy#R%C3%A9nyi_divergence
    fn renyi_divergence<S2, A2>(&self, q: &ArrayBase<S2, D>, alpha: A2) -> Result<A, EntropyWithAlphaError<A, A2>>
    where
        S2: Data<Elem = A>,
        A: Float,
        A2: Float;


    /// https://en.wikipedia.org/wiki/F-divergence
    fn total_variantion<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// https://en.wikipedia.org/wiki/F-divergence
    fn chi_alpha_divergence<S2, A2>(&self, q: &ArrayBase<S2, D>, alpha: A2) -> Result<A, EntropyWithAlphaError<A, A2>>
    where
        S2: Data<Elem = A>,
        A: Float,
        A2: Float;

    /// https://en.wikipedia.org/wiki/F-divergence
    fn kullback_leibert<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// https://en.wikipedia.org/wiki/F-divergence
    fn kullback_leibert_reversed<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// https://en.wikipedia.org/wiki/F-divergence
    fn jensen_shannon<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// https://en.wikipedia.org/wiki/F-divergence
    fn jeffreys_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;


    /// https://en.wikipedia.org/wiki/Bhattacharyya_distance
    fn bhattacharyya_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float
    {
        self.bhattacharyya_coefficient(q).map(|value| value.ln().neg())
    }

    /// https://en.wikipedia.org/wiki/Bhattacharyya_distance
    fn bhattacharyya_coefficient<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// https://en.wikipedia.org/wiki/Hellinger_distance
    fn hellinger_distance<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// https://en.wikipedia.org/wiki/F-divergence
    fn pearson_chi_square_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// https://en.wikipedia.org/wiki/F-divergence
    fn neyman_chi_square_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

}



// see: https://en.wikipedia.org/wiki/F-divergence

fn convert_float<A: Float>(f: f64) -> Result<A, EntropyError<A>> {
    if let Some(value) = A::from(f) {
        Ok(value)
    } else {
        Err(EntropyError::FloatCastError {
            typ: std::any::type_name::<f64>(),
            value: f
        })
    }
}

/// The base implementation for every discrete divergence form
fn generic_divergence<A, S1, S2, D, F>(
    p: &ArrayBase<S1, D>,
    q: &ArrayBase<S2, D>,
    f: F,
) -> Result<A, EntropyError<A>>
where
    S1: Data<Elem = A>,
    S2: Data<Elem = A>,
    A: Float,
    D: Dimension,
    F: Fn(A, A) -> A
{
    if p.is_empty() {
        return Err(MultiInputError::EmptyInput.into());
    }
    if p.shape() != q.shape() {
        return Err(MultiInputError::ShapeMismatch(
            ShapeMismatch {
                first_shape: p.shape().to_vec(),
                second_shape: q.shape().to_vec(),
            }
        ).into());
    }
    let mut temp: ArrayBase<_, D> = Array::zeros(p.raw_dim());
    Zip::from(&mut temp)
        .and(p)
        .and(q)
        .for_each(|result, &p, &q| {
            *result = f(p, q)
        });
    Ok(temp.sum())
}


impl<A, S, D> EntropyExt2<A, S, D> for ArrayBase<S, D>
where
    S: Data<Elem = A>,
    D: Dimension,
{
    fn renyi_entropy<A2>(&self, alpha: A2) -> Result<A, EntropyWithAlphaError<A, A2>>
    where
        A: Float,
        A2: Float
    {
        if alpha.is_one() {
            return Ok(self.entropy()?)
        }
        if self.is_empty() {
            Err(EntropyError::EmptyInput(EmptyInput).into())
        } else {
            let alpha: A = cast(alpha).ok_or_else(
                || EntropyWithAlphaError::CastError {
                    value: alpha.into(),
                    typ: std::any::type_name::<A>(),
                }
            )?;
            let entropy = self
                .mapv(|x| {
                    if x == A::zero() {
                        A::zero()
                    } else {
                        x.powf(alpha)
                    }
                })
                .sum()
                .ln();
            let one = A::one();
            Ok((one / (one - alpha)) * entropy)
        }
    }


    fn renyi_divergence<S2, A2>(&self, q: &ArrayBase<S2, D>, alpha: A2) -> Result<A, EntropyWithAlphaError<A, A2>>
    where
        S2: Data<Elem=A>,
        A: Float,
        A2: Float
    {

        if alpha.is_one() {
            return Ok(self.kl_divergence(q)?)
        }
        if self.is_empty() {
            return Err(MultiInputError::EmptyInput.into());
        }
        if self.shape() != q.shape() {
            return Err(MultiInputError::ShapeMismatch(
                ShapeMismatch {
                    first_shape: self.shape().to_vec(),
                    second_shape: q.shape().to_vec(),
                }
            ).into());
        }
        if alpha.is_zero() {
            let mut temp = Array::zeros(self.raw_dim());
            Zip::from(&mut temp)
                .and(self)
                .and(q)
                .for_each(|result, &p, &q| {
                    if !p.is_zero() {
                        *result = q;
                    }
                });
            return Ok(temp.sum().ln().neg())
        }

        if let Ok(comp) = convert_float::<A2>(0.5) {
            if alpha == comp {
                return Ok(
                    self.bhattacharyya_coefficient(q).map(
                        |value| {
                            let value = value.ln();
                            -(value + value)
                        }
                    )?
                )
            }
        }

        let alpha: A = cast(alpha).ok_or_else(
            || EntropyWithAlphaError::CastError {
                value: alpha.into(),
                typ: std::any::type_name::<A>(),
            }
        )?;

        let mut temp = Array::zeros(self.raw_dim());

        let factor = if alpha.is_zero() {
            Zip::from(&mut temp)
                .and(self)
                .and(q)
                .for_each(|result, &p, &q| {
                    *result = {
                        if p.is_zero() {
                            A::zero()
                        } else {
                            q
                        }
                    }
                });
            A::one().neg()
        } else {
            let alpha_minus_one = alpha - A::one();

            Zip::from(&mut temp)
                .and(self)
                .and(q)
                .for_each(|result, &p, &q| {
                    *result = {
                        if q.is_zero() {
                            A::zero()
                        } else {
                            p.powf(alpha) / q.powf(alpha_minus_one)
                        }
                    }
                });
            alpha_minus_one.recip()
        };
        let divergence = factor * temp.sum().ln();
        Ok(divergence)
    }

    #[inline]
    fn total_variantion<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        let two = convert_float(2.0)?;
        generic_divergence(self, q, |p, q| {
            (p - q).abs()
        }).map(|temp| {
            temp / two
        })
    }

    fn chi_alpha_divergence<S2, A2>(&self, q: &ArrayBase<S2, D>, alpha: A2) -> Result<A, EntropyWithAlphaError<A, A2>>
    where
        S2: Data<Elem=A>,
        A: Float,
        A2: Float
    {
        let alpha: A = cast(alpha).ok_or_else(
            || EntropyWithAlphaError::CastError {
                value: alpha.into(),
                typ: std::any::type_name::<A>(),
            }
        )?;
        if alpha < A::one() {
            return Err(
                EntropyError::IllegalParameterError {
                    name: "alpha",
                    value: alpha,
                    explanation: Some("Must be equal or greater than 1!")
                }.into()
            )
        }

        let temp = generic_divergence(self, q, |p, q| {
            if q.is_zero() {
                A::zero()
            } else {
                ((p - q) / q).abs().powf(alpha) * q
            }
        })?;
        let two: A = convert_float(2.0)?;
        Ok(temp / two)
    }

    #[inline]
    fn kullback_leibert<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float,
    {
        // generic_divergence(self, q, |p, q| {
        //     if p.is_zero() {
        //         A::zero()
        //     } else {
        //         p * (q / p).ln()
        //     }
        // }).map(A::neg)

        generic_divergence(self, q, |p, q| {
            if p.is_zero() {
                A::zero()
            } else {
                p * (p / q).ln()
            }
        })
    }

    #[inline]
    fn kullback_leibert_reversed<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        generic_divergence(self, q, |p, q| {
            if q.is_zero() {
                A::zero()
            } else {
                q * (p / q).ln()
            }
        }).map(A::neg)
    }

    #[inline]
    fn jensen_shannon<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        let two = convert_float(2.0)?;
        generic_divergence(self, q, |p, q| {
            let k = (p + q) / two;
            let a = p * (p / k).ln();
            let b = q * (p / k).ln();
            a + b
        }).map(|temp| {
            temp / two
        })
    }

    #[inline]
    fn jeffreys_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        generic_divergence(self, q, |p, q| {
            if q.is_zero() {
                A::zero()
            } else {
                (p - q) * (p/q).ln()
            }
        })
    }

    #[inline]
    fn bhattacharyya_coefficient<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        generic_divergence(self, q, |p, q| {
            (p * q).sqrt()
        })
    }

    #[inline]
    fn hellinger_distance<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        self.bhattacharyya_coefficient(q).map(|temp| A::one() - temp)
    }

    #[inline]
    fn pearson_chi_square_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        generic_divergence(self, q, |p, q| {
            if q.is_zero() {
                A::zero()
            } else {
                let tmp = p - q;
                (tmp * tmp) / q
            }
        })
    }

    #[inline]
    fn neyman_chi_square_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        generic_divergence(self, q, |p, q| {
            if q.is_zero() {
                A::zero()
            } else {
                let tmp = p - q;
                (tmp * tmp) / q
            }
        })
    }
}


#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use ndarray::{array, Array1};
    use ndarray_stats::{DeviationExt, EntropyExt};
    use crate::translate::entropies::EntropyExt2;

    #[test]
    fn hellinger_distance_works_ad_defined(){
        let x: Array1<f64> = array![
            0.0, 0.1, 0.5, 0.4
        ];
        let y: Array1<f64> = array![
            0.3, 0.6, 0.02, 0.08
        ];

        let a: Array1<f64> = array![
            1.0, 0.0
        ];
        let b: Array1<f64> = array![
            0.0, 1.0
        ];

        const MAX_DIST_UNOPT: f64 = std::f64::consts::SQRT_2;

        println!("{}", x.hellinger_distance(&y).unwrap());




        assert_relative_eq!(
            1.0,
            a.hellinger_distance(&b).unwrap()
        );

        assert_relative_eq!(
            1.0/std::f64::consts::SQRT_2 * a.l2_dist(&b).unwrap(),
            a.hellinger_distance(&b).unwrap()
        );

        assert_relative_eq!(
            a.kl_divergence(&b).unwrap(),
            a.kullback_leibert(&b).unwrap()
        );
    }


}












// if self.is_empty() {
//     return Err(MultiInputError::EmptyInput.into());
// }
// if self.shape() != q.shape() {
//     return Err(MultiInputError::ShapeMismatch(
//         ShapeMismatch {
//             first_shape: self.shape().to_vec(),
//             second_shape: q.shape().to_vec(),
//         }
//     ).into());
// }
// let mut temp = Array::zeros(self.raw_dim());
// Zip::from(&mut temp)
//     .and(self)
//     .and(q)
//     .for_each(|result, &p, &q| {
//         *result = {
//             (p * q).sqrt()
//         }
//     });
// Ok(temp.sum())