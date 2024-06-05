use std::iter::Map;
use std::ops::Range;
use std::slice::Iter;
use itertools::Itertools;

// src: https://github.com/piskvorky/gensim/blob/develop/gensim/_matutils.pyx


pub fn dirichlet_expectation_1d<'a>(alphas: &'a Vec<f64>) -> Map<Iter<'a, f64>, impl FnMut(&'a f64) -> f64 + 'a> {
    let psi_sum_alpha = statrs::function::gamma::digamma(alphas.iter().sum());
    alphas.iter().map(move |alpha| statrs::function::gamma::digamma(*alpha) - psi_sum_alpha)
}

pub fn dirichlet_expectation_2d<'a>(alphas: &'a Vec<Vec<f64>>) -> Map<Iter<'a, Vec<f64>>, impl FnMut(&'a Vec<f64>) -> Vec<f64> + 'a> {
    alphas.iter().map(|values| dirichlet_expectation_1d(values).collect_vec())
}

pub fn dot<'a>(a: &'a Vec<f64>, b: &'a Vec<Vec<f64>>) -> Map<Range<usize>, impl FnMut(usize) -> f64 + 'a > {
    assert!(!b.is_empty());
    (0..b[0].len()).map(|pos|
        a.iter().zip_eq(b.iter().map(|value| value[pos])).map(|(x, y)| y * x).sum::<f64>()
    )
}


pub fn transpose<'a>(v: &'a Vec<Vec<f64>>) -> Map<Range<usize>, impl FnMut(usize) -> Vec<f64> + 'a>
{
    assert!(!v.is_empty());
    (0..v[0].len()).map(|i| v.iter().map(|inner| inner[i]).collect::<Vec<_>>())
}