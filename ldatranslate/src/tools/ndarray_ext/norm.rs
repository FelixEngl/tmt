use ndarray::{Data, Dimension};

pub trait Norm<A, S, D>
where
    S: Data<Elem = A>,
    D: Dimension,
{
    /// rename of `norm_l2`
    fn norm(&self) -> A {
        self.norm_l2()
    }
    /// L-1 norm
    fn norm_l1(&self) -> A;
    /// L-2 norm
    fn norm_l2(&self) -> A;
    /// maximum norm
    fn norm_max(&self) -> A;
}

// impl<A, S, D> Norm<A, S, D> for ArrayBase<S, D>
// where
//     A: Float + ConstZero,
//     S: Data<Elem = A>,
//     D: Dimension,
// {
//     fn norm_l1(&self) -> A {
//         self.iter().map(|&x| x.abs()).sum()
//     }
//     fn norm_l2(&self) -> A {
//         self.iter().map(|&x| x * x).sum::<A>().sqrt()
//     }
//     fn norm_max(&self) -> A {
//         self.iter().fold(A::ZERO, |f, &val| {
//             let v = val.abs();
//             if f > v {
//                 f
//             } else {
//                 v
//             }
//         })
//     }
// }