use evalexpr::export::evalexpr_num::{Float, FromPrimitive};
use itertools::Itertools;
use ndarray::{Array, ArrayBase, Data, Dimension, Zip};
use ndarray_stats::EntropyExt;
use ndarray_stats::errors::{EmptyInput, MultiInputError, ShapeMismatch};
use num::{cast};
use pyo3::pyclass;
use sealed::sealed;
use ldatranslate_toolkit::partial_ord_iterator::PartialOrderIterator;
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
use ldatranslate_topicmodel::dictionary::metadata::ex::MetaField;
use crate::translate::entropies::errors::*;

register_python! {
    enum FDivergence;
}


#[derive(Clone, Debug)]
pub struct FDivergenceCalculator {
    pub fdivergence: FDivergence,
    pub alpha: Option<f64>,
    pub target_fields: Option<Vec<DictMetaTagIndex>>,
    pub invert_target_fields: bool
}

impl FDivergenceCalculator {


    pub fn calculate<S1, S2, A, D>(&self, p: &ArrayBase<S1, D>, q: &ArrayBase<S2, D>) -> Result<A, EntropyWithAlphaError<A, f64>>
    where
        S1: Data<Elem=A>,
        S2: Data<Elem=A>,
        A: Float + FromPrimitive,
        D: Dimension
    {
        match self.fdivergence {
            FDivergence::Renyi => {
                p.renyi_divergence(q, self.alpha.unwrap_or(1.0))
            }
            FDivergence::Total => {
                Ok(p.total_variantion(q)?)
            }
            FDivergence::ChiAlpha => {
                if let Some(alpha) = self.alpha {
                    p.chi_alpha_divergence(q, alpha)
                } else {
                    Ok(p.total_variantion(q)?)
                }
            }
            FDivergence::KL => {
                Ok(p.kullback_leibert(q)?)
            }
            FDivergence::KLReversed => {
                Ok(p.kullback_leibert_reversed(q)?)
            }
            FDivergence::JensenShannon => {
                Ok(p.jensen_shannon(q)?)
            }
            FDivergence::Jeffrey => {
                Ok(p.jeffreys_divergence(q)?)
            }
            FDivergence::Bhattacharyya => {
                Ok(p.bhattacharyya_divergence(q)?)
            }
            FDivergence::Hellinger => {
                Ok(p.hellinger_distance(q)?)
            }
            FDivergence::PearsonChiSquare => {
                Ok(p.pearson_chi_square_divergence(q)?)
            }
            FDivergence::NeymanChiSquare => {
                Ok(p.neyman_chi_square_divergence(q)?)
            }
        }
    }

    pub fn new(fdivergence: FDivergence, alpha: Option<f64>, target_fields: Option<Vec<DictMetaTagIndex>>, invert_target_fields: bool) -> Self {
        Self { fdivergence, alpha, target_fields, invert_target_fields }
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyo3::pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FDivergence {
    Renyi,
    Total,
    ChiAlpha,
    KL,
    KLReversed,
    JensenShannon,
    Jeffrey,
    Bhattacharyya,
    Hellinger,
    PearsonChiSquare,
    NeymanChiSquare
}

#[sealed]
pub trait FDivergenceExt<A, S, D>
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
        A: Float + FromPrimitive,
        A2: Float + FromPrimitive;


    /// In probability theory, the total variation distance is a distance measure for probability
    /// distributions. It is an example of a statistical distance metric, and is sometimes
    /// called the statistical distance, statistical difference or variational distance.
    ///
    /// This is also the `chi_alpha_divergence` where alpha = 1
    ///
    /// See also:
    /// [F-Divergence](https://en.wikipedia.org/wiki/F-divergence)
    /// [Total variation distance of probability measures](https://en.wikipedia.org/wiki/Total_variation_distance_of_probability_measures)
    fn total_variantion<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// Requires that alpha >= 1.
    /// Basically a variant of the `total_variantion` where the alpha is a shape parameter.
    ///
    /// When alpha = 1, it results in the `total_variantion`
    ///
    /// See also:
    /// [F-Divergence](https://en.wikipedia.org/wiki/F-divergence)
    fn chi_alpha_divergence<S2, A2>(&self, q: &ArrayBase<S2, D>, alpha: A2) -> Result<A, EntropyWithAlphaError<A, A2>>
    where
        S2: Data<Elem = A>,
        A: Float + FromPrimitive,
        A2: Float + FromPrimitive;

    /// Requires a relative entropy with absolute continuity, where Q(x) = 0 implies P(x) = 0.
    /// Otherwise results in infinity.
    ///
    /// If P(x) = 0, it returns 0.
    ///
    /// See also:
    /// [F-Divergence](https://en.wikipedia.org/wiki/F-divergence)
    fn kullback_leibert<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// Requires a relative entropy with absolute continuity, where Q(x) = 0 implies P(x) = 0.
    /// Otherwise results in infinity.
    ///
    /// If P(x) = 0, results in 0
    ///
    /// See also:
    /// [F-Divergence](https://en.wikipedia.org/wiki/F-divergence)
    fn kullback_leibert_reversed<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// This implementation depends on the assumption that we have a relative entropy with
    /// absolute continuity, where Q(x) = 0 implies P(x) = 0.
    ///
    /// If Q(x) = 0, it short-circuit to 0, even if P(x) != 0.
    ///
    /// See also:
    /// [F-Divergence](https://en.wikipedia.org/wiki/F-divergence),
    /// [Jensen–Shannon divergence](https://en.wikipedia.org/wiki/Jensen%E2%80%93Shannon_divergence)
    fn jensen_shannon<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// [F-Divergence](https://en.wikipedia.org/wiki/F-divergence)
    fn jeffreys_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;


    /// https://en.wikipedia.org/wiki/Bhattacharyya_distance
    fn bhattacharyya_coefficient<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// The Bhattacharyya distance is a quantity which represents a notion of similarity between
    /// two probability distributions.[1] It is closely related to the Bhattacharyya coefficient,
    /// which is a measure of the amount of overlap between two statistical samples or populations.
    ///
    /// It is not a metric, despite being named a "distance",
    /// since it does not obey the triangle inequality.
    ///
    /// https://en.wikipedia.org/wiki/Bhattacharyya_distance
    fn bhattacharyya_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float
    {
        self.bhattacharyya_coefficient(q).map(|value| value.ln().neg())
    }

    /// The squared hellinger distance is related to the euclidean distance.
    /// But can be implemented by using the bhattacharyya coefficient (BC).
    /// https://en.wikipedia.org/wiki/Hellinger_distance
    fn hellinger_distance<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float
    {
        self.bhattacharyya_coefficient(q).map(|temp| A::one() - temp)
    }

    /// [F-Divergence](https://en.wikipedia.org/wiki/F-divergence)
    fn pearson_chi_square_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

    /// [F-Divergence](https://en.wikipedia.org/wiki/F-divergence)
    fn neyman_chi_square_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem = A>,
        A: Float;

}



// see: [F-Divergence](https://en.wikipedia.org/wiki/F-divergence)

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


#[sealed]
impl<A, S, D> FDivergenceExt<A, S, D> for ArrayBase<S, D>
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
        A: Float + FromPrimitive,
        A2: Float + FromPrimitive
    {
        if alpha.is_sign_negative() {
            return Err(EntropyWithAlphaError::negative_alpha(stringify!(renyi_divergence)));
        }

        if alpha.is_nan() {
            return Err(EntropyWithAlphaError::nan(stringify!(renyi_divergence)));
        }

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

        if alpha.is_infinite() {
            //  the log of the maximum ratio of the probabilities.
            // todo: is this really what they mean?
            return Ok(
                self.iter().zip_eq(q.iter()).filter_map(|(&p, &q)|{
                    if p.is_zero() || q.is_zero() {
                        None
                    } else {
                        Some((p*p)/q)
                    }
                }).max_partial_filtered().unwrap_or_else(A::zero)
            )
        }

        if Some(alpha) == A2::from_f64(0.5) {
            return Ok(
                self.bhattacharyya_coefficient(q).map(
                    |value| {
                        let value = value.ln();
                        -(value + value)
                    }
                )?
            )
        }

        let mut temp = Array::zeros(self.raw_dim());

        if Some(alpha) == A2::from_f64(2.0) {
            // the log of the expected ratio of the probabilities
            // this is 1/1 * log(sum(p^2/q))
            Zip::from(&mut temp)
                .and(self)
                .and(q)
                .for_each(|result, &p, &q| {
                    if !p.is_zero() && !q.is_zero() {
                        *result = (p*p)/q;
                    }
                });
            return Ok(temp.sum().ln())
        }

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
            let alpha: A = cast(alpha).ok_or_else(
                || EntropyWithAlphaError::CastError {
                    value: alpha.into(),
                    typ: std::any::type_name::<A>(),
                }
            )?;

            let alpha_minus_one = alpha - A::one();

            Zip::from(&mut temp)
                .and(self)
                .and(q)
                .for_each(|result, &p, &q| {
                    *result = {
                        if p.is_zero() || q.is_zero() {
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
        if alpha.is_one() {
            return Ok(self.total_variantion(q)?)
        }
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
        generic_divergence(self, q, |p, q| {
            if p.is_zero() {
                A::zero()
            } else {
                p * (q / p).ln()
            }
        }).map(A::neg)
    }

    #[inline]
    fn kullback_leibert_reversed<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        generic_divergence(self, q, |p, q| {
            if p.is_zero() {
                A::zero()
            } else {
                q * (q / p).ln()
            }
        })
    }

    #[inline]
    fn jensen_shannon<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        let two = convert_float(2.0)?;
        generic_divergence(self, q, |p, q| {
            if q.is_zero() {
                A::zero()
            } else {
                let k = (p + q) / two;
                let a = if p.is_zero() {
                    A::zero()
                } else {
                    p * (p / k).ln()
                };
                a + (q * (q / k).ln())
            }
        }).map(|temp| {
            temp / two
        })
    }

    /// Requires a relative entropy with absolute continuity, where Q(x) = 0 implies P(x) = 0.
    /// Otherwise results in infinity.
    /// If P(x) = 0, results in 0
    #[inline]
    fn jeffreys_divergence<S2>(&self, q: &ArrayBase<S2, D>) -> Result<A, EntropyError<A>>
    where
        S2: Data<Elem=A>,
        A: Float
    {
        generic_divergence(self, q, |p, q| {
            if p.is_zero() {
                A::zero()
            } else {
                (p - q) * (q/p).ln()
            }
        }).map(A::neg)
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

    /// This implementation depends on the assumption that we have a relative entropy with
    /// absolute continuity, where Q(x) = 0 implies P(x) = 0.
    ///
    /// If Q(x) = 0, it short-circuit to 0, even if P(x) != 0.
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
            if p.is_zero() {
                A::zero()
            } else {
                let tmp = p - q;
                (tmp * tmp) / p
            }
        })
    }
}


#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use ndarray::{array, Array1};
    use ndarray_stats::{DeviationExt, EntropyExt};
    use crate::translate::entropies::{FDivergenceExt};

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

        println!("total_variantion {}", x.total_variantion(&y).unwrap());
        println!("kullback_leibert {}", x.kullback_leibert(&y).unwrap());
        println!("kullback_leibert_reversed {}", x.kullback_leibert_reversed(&y).unwrap());
        println!("jensen_shannon {}", x.jensen_shannon(&y).unwrap());
        println!("jeffreys_divergence {}", x.jeffreys_divergence(&y).unwrap());
        println!("bhattacharyya_divergence {}", x.bhattacharyya_divergence(&y).unwrap());
        println!("hellinger_distance {}", x.hellinger_distance(&y).unwrap());
        println!("pearson_chi_square_divergence {}", x.pearson_chi_square_divergence(&y).unwrap());
        println!("neyman_chi_square_divergence {}", x.neyman_chi_square_divergence(&y).unwrap());
        println!("Rényi divergence {}", x.renyi_divergence(&y, 2.0).unwrap());

        // Rényi divergence 2.675297414630402

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