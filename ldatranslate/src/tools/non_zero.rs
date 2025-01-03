use ndarray::{Array1};
use ldatranslate_toolkit::partial_ord_iterator::PartialOrderIterator;

pub fn make_positive_only(target: &mut [f64]) {
    let min = target.iter().min_partial_filtered().copied();
    if let Some(min) = min {
        if min.is_sign_negative() {
            let min = min.abs();
            for value in target.iter_mut() {
                *value = (*value + min).abs();
            }
        }
    }
}

pub fn make_positive_only_arr(target: &mut Array1<f64>) {
    let min = target.iter().min_partial_filtered().copied();
    if let Some(min) = min {
        if min.is_sign_negative() {
            let min = min.abs();
            for value in target.iter_mut() {
                *value = (*value + min).abs();
            }
        }
    }
}