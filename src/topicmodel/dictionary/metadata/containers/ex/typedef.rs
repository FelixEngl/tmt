macro_rules! make_storage_type_def {
    (set: $name: ident: $ty: ty; $($tt:tt)*) => {
        paste::paste! {
            pub type [<$name:camel StoreType>]<'a> = &'a $crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$ty>;
        }
        $crate::topicmodel::dictionary::metadata::ex::typedef::make_storage_type_def!($($tt)*);
    };
    (interned: $name: ident: $ty: ty; $($tt:tt)*) => {
        paste::paste! {
            pub type [<$name:camel StoreType>]<'a> = (&'a $crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$ty>, Vec<(&'a str, u32)>);
        }
        $crate::topicmodel::dictionary::metadata::ex::typedef::make_storage_type_def!($($tt)*);
    };
    (voc: $name: ident: $ty: ty; $($tt:tt)*) => {
        paste::paste! {
            pub type [<$name:camel StoreType>]<'a> = (&'a $crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$ty>, Vec<(&'a str, u32)>);
        }
        $crate::topicmodel::dictionary::metadata::ex::typedef::make_storage_type_def!($($tt)*);
    };
    ($(;)?) => {}
}

pub(super) use make_storage_type_def;