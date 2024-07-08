use std::cmp::Ordering;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, PartialOrd)]
#[error("The partial_cmp of {candidate} and {cause} results in an invalid comparison result!")]
pub struct PartialOrderIteratorError<T> {
    /// The last candidate for the min/max
    pub candidate: T,
    /// The cause og the
    pub cause: T
}


/// Allows to find the min and max on partial iterators.
pub trait PartialOrderIterator : Iterator {
    /// Returns the maximum element of an iterator of partial comparable elements.
    ///
    /// If several elements are equally maximum, the last element is returned.
    /// If the iterator is empty, None is returned.
    ///
    /// If one of the comparisons results in a None, an error is returned with
    /// the maximum before the error as `candidate` and the value that caused the error as `cause`.
    /// If the first value causes the error, the current value may also be a non-comparable.
    fn max_partial(self) -> Result<Option<Self::Item>, PartialOrderIteratorError<Self::Item>>;

    /// Returns the minimum element of an iterator of partial comparable elements.
    ///
    /// If several elements are equally minimum, the last element is returned.
    /// If the iterator is empty, None is returned.
    ///
    /// If one of the comparisons results in a None, an error is returned with
    /// the minimum before the error as `candidate` and the value that caused the error as `cause`.
    /// If the first value causes the error, the current value may also be a non-comparable.
    #[allow(dead_code)]
    fn min_partial(self) -> Result<Option<Self::Item>, PartialOrderIteratorError<Self::Item>>;

    /// Returns the maximum element of an iterator of partial comparable elements.
    ///
    /// If several elements are equally maximum, the last element is returned.
    /// If the iterator is empty, None is returned.
    ///
    /// If one of the comparisons results in a None, the causing value is ignored.
    fn max_partial_filtered(self) -> Option<Self::Item>;

    /// Returns the minimum element of an iterator of partial comparable elements.
    ///
    /// If several elements are equally maximum, the last element is returned.
    /// If the iterator is empty, None is returned.
    ///
    /// If one of the comparisons results in a None, the causing value is ignored.
    fn min_partial_filtered(self) -> Option<Self::Item>;
}



macro_rules! partial_min_max_alg {
    ($self: expr, $target: path) => {
        if let Some(mut value) = $self.next() {
            while let Some(other) = $self.next() {
                // We invert the comparison to get the other two values to emulate the normal max behaviour.
                match PartialOrd::partial_cmp(&value, &other) {
                    Some($target) => {},
                    Some(_) => value = other,
                    None => {
                        return Err(PartialOrderIteratorError{
                            candidate: value,
                            cause: other
                        })
                    },
                }
            }
            Ok(Some(value))
        } else {
            Ok(None)
        }
    };
    (filtered $self: expr, $target: path) => {
        if let Some(mut value) = $self.next() {
            let mut value_before = None;
            while let Some(other) = $self.next() {
                // We invert the comparison to get the other two values to emulate the normal max behaviour.
                match PartialOrd::partial_cmp(&value, &other) {
                    Some($target) => {}
                    Some(_) => value = other,
                    None => {
                        if let Some(previous_other) = value_before {
                            value_before = match PartialOrd::partial_cmp(&previous_other, &other) {
                                None => {
                                    Some(other)
                                }
                                Some($target) => {
                                    value = previous_other;
                                    None
                                },
                                _ => {
                                    value = other;
                                    None
                                }
                            }
                        } else {
                            value_before = Some(other);
                        }
                    }
                }
            }
            Some(value)
        } else {
            None
        }
    };
}

impl<I, T> PartialOrderIterator for I
    where
        Self: Sized,
        I: Iterator<Item = T>,
        T: PartialOrd
{
    fn max_partial(mut self) -> Result<Option<Self::Item>, PartialOrderIteratorError<Self::Item>> {
        partial_min_max_alg!(self, Ordering::Greater)
    }

    fn min_partial(mut self) -> Result<Option<Self::Item>, PartialOrderIteratorError<Self::Item>> {
        partial_min_max_alg!(self, Ordering::Less)
    }

    fn max_partial_filtered(mut self) -> Option<Self::Item> {
        partial_min_max_alg!(filtered self, Ordering::Greater)
    }

    fn min_partial_filtered(mut self) -> Option<Self::Item> {
        partial_min_max_alg!(filtered self, Ordering::Less)
    }
}





#[cfg(test)]
mod test {
    use crate::toolkit::partial_ord_iterator::{PartialOrderIterator};

    #[test]
    pub fn can_work_with_integers() {
        let data1 = vec![10, 1,2,3,4,5,6,7,8,9, 11, 12];
        assert_eq!(Ok(Some(&1)), data1.iter().min_partial());
        assert_eq!(Ok(Some(&12)), data1.iter().max_partial());
        assert_eq!(Some(&1), data1.iter().min_partial_filtered());
        assert_eq!(Some(&12), data1.iter().max_partial_filtered());
    }

    #[test]
    pub fn can_work_with_floats1() {
        let data2 = vec![f64::NAN, f64::NAN, 10., 1.,2., f64::NAN,3.,4.,5.,6., f64::NAN,7.,8.,9., f64::NAN, 11., 12., f64::NAN];
        let result = data2.iter().min_partial().expect_err("Expected an error!");
        assert!(result.candidate.is_nan(), "current not Nan");
        assert!(result.cause.is_nan());
        let result = data2.iter().max_partial().expect_err("Expected an error!");
        assert!(result.candidate.is_nan());
        assert!(result.cause.is_nan());
        assert_eq!(Some(&1.0), data2.iter().min_partial_filtered());
        assert_eq!(Some(&12.0), data2.iter().max_partial_filtered());
    }

    #[test]
    pub fn can_work_with_floats2() {
        let data3 = vec![f64::NAN, 10., 1.,2., f64::NAN,3.,4.,5.,6., f64::NAN,7.,8.,9., f64::NAN, 11., 12., f64::NAN];
        let result = data3.iter().min_partial().expect_err("Expected an error!");
        assert!(result.candidate.is_nan());
        assert_eq!(&10.0, result.cause);
        let result = data3.iter().max_partial().expect_err("Expected an error!");
        assert!(result.candidate.is_nan());
        assert_eq!(&10.0, result.cause);
        assert_eq!(Some(&1.0), data3.iter().min_partial_filtered());
        assert_eq!(Some(&12.0), data3.iter().max_partial_filtered());
    }
}