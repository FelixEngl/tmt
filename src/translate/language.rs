use std::ops::Deref;

/// Allows to differentiate the source of the object regarding a language
#[derive(Copy, Clone, Debug)]
pub(super) enum LanguageOrigin<T> {
    Origin(T),
    Target(T)
}

impl<T> Deref for LanguageOrigin<T> {
    type Target = T;

    fn deref(&self) -> &<Self as Deref>::Target {
        match self {
            Self::Origin(value) => {value}
            Self::Target(value) => {value}
        }
    }
}