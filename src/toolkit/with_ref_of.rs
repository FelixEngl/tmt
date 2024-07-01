use std::iter::FusedIterator;

pub trait SupportsWithRef {
    // fn with_ref<V>(self, value: &V) -> WithRef<Self, V> where Self: Iterator + Sized;
    fn with_value<V>(self, value: V) -> WithValue<Self, V> where Self: Iterator + Sized;
}

impl<I: Iterator> SupportsWithRef for I {
    // fn with_ref<V>(self, value: &V) -> WithRef<Self, V> where Self: Iterator + Sized {
    //     WithRef::new(self, value)
    // }

    fn with_value<V>(self, value: V) -> WithValue<Self, V> where Self: Iterator + Sized {
        WithValue::new(self, value)
    }
}

// #[derive(Debug)]
// pub struct WithRef<'a, I, V> {
//     iter: I,
//     value: &'a V
// }
//
// impl<'a, I, V> WithRef<'a, I, V> {
//     pub(crate) fn new(iter: I, value: &'a V) -> Self {
//         Self { iter, value }
//     }
// }
//
//
// impl<'a, I: Iterator, V> Iterator for WithRef<'a, I, V> {
//     type Item = (&'a V, I::Item);
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next().map(|value| (self.value, value))
//     }
// }
//
// impl<'a, I: DoubleEndedIterator, V: 'a> DoubleEndedIterator for WithRef<'a, I, V>
// {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|value| (self.value, value))
//     }
// }
//
// impl<'a, I: ExactSizeIterator, V> ExactSizeIterator for WithRef<'a, I, V>
// {
//     fn len(&self) -> usize {
//         self.iter.len()
//     }
// }
//
// impl<'a, I: FusedIterator, V> FusedIterator for WithRef<'a, I, V>
// {
// }



#[derive(Debug)]
pub struct WithValue<I, V> {
    iter: I,
    value: V
}

impl<I, V> WithValue<I, V> {
    pub(crate) fn new(iter: I, value: V) -> Self {
        Self { iter, value }
    }
}


impl<I: Iterator, V: Clone> Iterator for WithValue<I, V> {
    type Item = (V, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|value| (self.value.clone(), value))
    }
}

impl<I: DoubleEndedIterator, V: Clone> DoubleEndedIterator for WithValue< I, V>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|value| (self.value.clone(), value))
    }
}

impl<I: ExactSizeIterator, V: Clone> ExactSizeIterator for WithValue< I, V>
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I: FusedIterator, V: Clone> FusedIterator for WithValue< I, V>
{
}
