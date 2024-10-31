use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use serde::de::{Error, Visitor};
use string_interner::{Symbol};

pub trait TypeSafeSymbol<S>: Symbol + From<S> where S: Symbol {}

pub struct SymbolVisitor<S: Symbol>{_phantom: PhantomData<S>}
impl<S> Default for SymbolVisitor<S> where S: Symbol {
    #[inline(always)]
    fn default() -> Self {
        Self{_phantom: PhantomData}
    }
}

impl<'de, S> Visitor<'de> for SymbolVisitor<S> where S: Symbol {
    type Value = S;
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "Expected an interned symbol.")
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(
            S::try_from_usize(
                match v.try_into() {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(E::custom(err.to_string()))
                    }
                }
            ).expect("This should never fail!")
        )
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(
            S::try_from_usize(
                match v.try_into() {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(E::custom(err.to_string()))
                    }
                }
            ).expect("This should never fail!")
        )
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(S::try_from_usize(v as usize).expect("This should never fail!"))
    }
}

#[macro_export]
macro_rules! create_interned_typesafe_symbol {
    ($($name: ident),+ $(,)?) => {
        $(
            #[derive(Hash)]
            #[repr(transparent)]
            pub struct $name<S>(S) where S: string_interner::Symbol;

            impl<S> $name<S> where S: string_interner::Symbol {
                #[inline(always)]
                pub fn into_inner(self) -> S {
                    self.0
                }
            }

            impl<S> Debug for $name<S>
            where
                S: string_interner::Symbol + Debug,
            {
                delegate::delegate! {
                    to self.0 {
                        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
                    }
                }
            }

            impl<S> string_interner::Symbol for $name<S>
            where
                S: string_interner::Symbol,
            {
                #[inline(always)]
                fn try_from_usize(index: usize) -> Option<Self> {
                    S::try_from_usize(index).map(|value| Self(value))
                }

                #[inline(always)]
                fn to_usize(self) -> usize {
                    self.0.to_usize()
                }
            }

            impl<S> tinyset::set64::Fits64 for $name<S>
            where
                S: string_interner::Symbol,
            {
                #[inline(always)]
                unsafe fn from_u64(x: u64) -> Self {
                    S::try_from_usize(x as usize).map(|value| Self(value)).unwrap()
                }

                #[inline(always)]
                fn to_u64(self) -> u64 {
                    self.0.to_usize() as u64
                }
            }

            impl<S> Copy for $name<S> where S: string_interner::Symbol {}

            impl<S> Clone for $name<S>
            where
                S: string_interner::Symbol
            {
                #[inline(always)]
                fn clone(&self) -> Self {
                    Self(self.0.clone())
                }
            }

            impl<S> Eq for $name<S> where S: string_interner::Symbol {}

            impl<S> PartialEq<Self> for $name<S>
            where
                S: string_interner::Symbol,
            {
                #[inline(always)]
                fn eq(&self, other: &Self) -> bool {
                    self.0.eq(&other.0)
                }
            }

            impl<S> From<S> for $name<S>
            where
                S: string_interner::Symbol
            {
                #[inline(always)]
                fn from(value: S) -> Self {
                    Self(value)
                }
            }

            impl<S> Display for $name<S>
            where
                S: string_interner::Symbol
            {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, stringify!($name))?;
                    write!(f, "{}", self.to_usize())
                }
            }


            impl<S> $crate::toolkit::typesafe_interner::TypeSafeSymbol<S> for $name<S> where S: Symbol {}



            impl serde::Serialize for $name<string_interner::symbol::SymbolU16> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer
                {
                    serializer.serialize_u16(self.to_usize() as u16)
                }
            }

            impl<'de> serde::Deserialize<'de> for $name<string_interner::symbol::SymbolU16>  {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>
                {
                    deserializer.deserialize_u16($crate::toolkit::typesafe_interner::SymbolVisitor::default())
                }
            }

            impl serde::Serialize for $name<string_interner::symbol::SymbolU32> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer
                {
                    serializer.serialize_u32(self.to_usize() as u32)
                }
            }

            impl<'de> serde::Deserialize<'de> for $name<string_interner::symbol::SymbolU32>  {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>
                {
                    deserializer.deserialize_u32($crate::toolkit::typesafe_interner::SymbolVisitor::default())
                }
            }

            impl serde::Serialize for $name<string_interner::symbol::SymbolUsize> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer
                {
                    serializer.serialize_u64(self.to_usize() as u64)
                }
            }

            impl<'de> serde::Deserialize<'de> for $name<string_interner::symbol::SymbolUsize>  {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>
                {
                    deserializer.deserialize_u64($crate::toolkit::typesafe_interner::SymbolVisitor::default())
                }
            }

            impl serde::Serialize for $name<usize> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer
                {
                    serializer.serialize_u64(self.to_usize() as u64)
                }
            }

            impl<'de> serde::Deserialize<'de> for $name<usize>  {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>
                {
                    deserializer.deserialize_u64($crate::toolkit::typesafe_interner::SymbolVisitor::default())
                }
            }


            paste::paste!{
                pub type [<$name Symbol>] = $name<string_interner::DefaultSymbol>;
                pub type [<$name Backend>]<S> = string_interner::backend::StringBackend<$name<S>>;
                pub type [<Default $name Backend>] = [<$name Backend>]<string_interner::DefaultSymbol>;
                pub type [<Default $name StringInterner>]<H = string_interner::DefaultHashBuilder> = string_interner::DefaultStringInterner<[<Default $name Backend>], H>;
            }
        )+
    };
}


create_interned_typesafe_symbol! {
    DictionaryOrigin,
    Tag,
    Inflected,
    Abbreviation,
    UnalteredVoc,
    OriginalEntry,
    AnyId,
    ContextualInformation,
    Unclassified
}
