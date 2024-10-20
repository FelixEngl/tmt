use derive_more::From;

#[derive(From, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Name<I>(pub I);