use std::fmt::{Formatter};
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
            pub struct $name<S>(S) where S: $crate::exports::string_interner::Symbol;

            impl<S> $name<S> where S: $crate::exports::string_interner::Symbol {
                #[inline(always)]
                pub fn into_inner(self) -> S {
                    self.0
                }
            }

            impl<S> std::fmt::Debug for $name<S>
            where
                S: $crate::exports::string_interner::Symbol + std::fmt::Debug,
            {
                delegate::delegate! {
                    to self.0 {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
                    }
                }
            }

            impl<S> $crate::exports::string_interner::Symbol for $name<S>
            where
                S: $crate::exports::string_interner::Symbol,
            {
                #[inline(always)]
                fn try_from_usize(index: usize) -> Option<Self> {
                    S::try_from_usize(index).map(|value| Self(value))
                }

                #[inline(always)]
                fn to_usize(self) -> usize {
                    use $crate::exports::string_interner::symbol::Symbol;
                    self.0.to_usize()
                }
            }

            impl<S> $crate::exports::tinyset::set64::Fits64 for $name<S>
            where
                S: $crate::exports::string_interner::Symbol,
            {
                #[inline(always)]
                unsafe fn from_u64(x: u64) -> Self {
                    S::try_from_usize(x as usize).map(|value| Self(value)).unwrap()
                }

                #[inline(always)]
                fn to_u64(self) -> u64 {
                    use $crate::exports::string_interner::symbol::Symbol;
                    self.0.to_usize() as u64
                }
            }

            impl<S> Copy for $name<S> where S: $crate::exports::string_interner::Symbol {}

            impl<S> Clone for $name<S>
            where
                S: $crate::exports::string_interner::Symbol
            {
                #[inline(always)]
                fn clone(&self) -> Self {
                    Self(self.0.clone())
                }
            }

            impl<S> Eq for $name<S> where S: $crate::exports::string_interner::Symbol {}

            impl<S> PartialEq<Self> for $name<S>
            where
                S: $crate::exports::string_interner::Symbol,
            {
                #[inline(always)]
                fn eq(&self, other: &Self) -> bool {
                    self.0.eq(&other.0)
                }
            }

            impl<S> From<S> for $name<S>
            where
                S: $crate::exports::string_interner::Symbol
            {
                #[inline(always)]
                fn from(value: S) -> Self {
                    Self(value)
                }
            }

            impl<S> std::fmt::Display for $name<S>
            where
                S: $crate::exports::string_interner::Symbol
            {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use $crate::exports::string_interner::symbol::Symbol;
                    write!(f, stringify!($name))?;
                    write!(f, "{}", self.to_usize())
                }
            }


            impl<S> $crate::typesafe_interner::TypeSafeSymbol<S> for $name<S> where S: $crate::exports::string_interner::Symbol {}



            impl $crate::exports::serde::Serialize for $name<$crate::exports::string_interner::symbol::SymbolU16> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: $crate::exports::serde::Serializer
                {
                    use $crate::exports::string_interner::symbol::Symbol;
                    serializer.serialize_u16(self.to_usize() as u16)
                }
            }

            impl<'de> $crate::exports::serde::Deserialize<'de> for $name<$crate::exports::string_interner::symbol::SymbolU16>  {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: $crate::exports::serde::Deserializer<'de>
                {
                    deserializer.deserialize_u16($crate::typesafe_interner::SymbolVisitor::default())
                }
            }

            impl $crate::exports::serde::Serialize for $name<$crate::exports::string_interner::symbol::SymbolU32> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: $crate::exports::serde::Serializer
                {
                    use $crate::exports::string_interner::symbol::Symbol;
                    serializer.serialize_u32(self.to_usize() as u32)
                }
            }

            impl<'de> $crate::exports::serde::Deserialize<'de> for $name<$crate::exports::string_interner::symbol::SymbolU32>  {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: $crate::exports::serde::Deserializer<'de>
                {
                    deserializer.deserialize_u32($crate::typesafe_interner::SymbolVisitor::default())
                }
            }

            impl $crate::exports::serde::Serialize for $name<$crate::exports::string_interner::symbol::SymbolUsize> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: $crate::exports::serde::Serializer
                {
                    use $crate::exports::string_interner::symbol::Symbol;
                    serializer.serialize_u64(self.to_usize() as u64)
                }
            }

            impl<'de> $crate::exports::serde::Deserialize<'de> for $name<$crate::exports::string_interner::symbol::SymbolUsize>  {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: $crate::exports::serde::Deserializer<'de>
                {
                    deserializer.deserialize_u64($crate::typesafe_interner::SymbolVisitor::default())
                }
            }

            impl $crate::exports::serde::Serialize for $name<usize> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: $crate::exports::serde::Serializer
                {
                    use $crate::exports::string_interner::symbol::Symbol;
                    serializer.serialize_u64(self.to_usize() as u64)
                }
            }

            impl<'de> $crate::exports::serde::Deserialize<'de> for $name<usize>  {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: $crate::exports::serde::Deserializer<'de>
                {
                    deserializer.deserialize_u64($crate::typesafe_interner::SymbolVisitor::default())
                }
            }


            $crate::exports::paste::paste!{
                pub type [<$name Symbol>] = $name<$crate::exports::string_interner::DefaultSymbol>;
                pub type [<$name Backend>]<S=$crate::exports::string_interner::DefaultSymbol> = $crate::exports::string_interner::backend::StringBackend<$name<S>>;
                pub type [<$name StringInterner>]<S=$crate::exports::string_interner::DefaultSymbol, H = $crate::exports::string_interner::DefaultHashBuilder> = $crate::exports::string_interner::DefaultStringInterner<[<$name Backend>]<S>, H>;
            }
        )+
    };
}


