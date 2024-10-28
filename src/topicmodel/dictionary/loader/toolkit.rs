
macro_rules! vec_to_option_or_panic {
    ($vec: expr $(, $($tt:tt)*)?) => {
        {
            match $vec.len() {
                0 => None,
                1 => Some($vec.into_iter().exactly_one().expect("This should never happen!")),
                other => panic!($($($tt)*)?)
            }
        }
    };
}

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use itertools::EitherOrBoth;
pub(super) use vec_to_option_or_panic;

macro_rules! replace_none_or_panic {
    ($opt: expr, $value: expr $(, $($tt:tt)*)?) => {
        if let Some(replaced) = $opt.replace($value) {
            panic!($($($tt)*)?);
        }
    };
}

pub(super) use replace_none_or_panic;


#[cfg(test)]
pub mod for_test_only {
    use std::sync::LazyLock;
    use convert_case::Case::Pascal;
    use convert_case::Casing;
    use regex::Regex;

    pub fn create_enum_definition<I: IntoIterator<Item = V>, V: AsRef<str>>(name: &str, iter: I) -> String {

        fn create_enum_name(name: &str) -> String {
            static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("[^a-zA-Z0-9]").unwrap());
            let name = name.trim();
            let result = REGEX.replace_all(name, "_").to_case(Pascal);
            if result.starts_with(|c| matches!(c, 'a'..'z' | 'A'..'Z')) {
                result
            } else {
                const FIXED_PREFIX: &str = "ZZZ_FIXED_";
                let mut s = String::with_capacity(result.len() + FIXED_PREFIX.len());
                s.push_str(FIXED_PREFIX);
                s.push_str(&result);
                s
            }
        }
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "#[derive(Copy, Clone, Debug, strum::Display, strum::EnumString, Eq, PartialEq, Hash)]\n").unwrap();
        write!(s, "pub enum {} {{\n", name.to_case(Pascal)).unwrap();
        for original_name in iter {
            let original_name = original_name.as_ref().trim();
            let enum_name = create_enum_name(original_name);
            write!(s, "    #[strum(to_string = \"{}\")]\n", original_name).unwrap();
            write!(s, "    {},\n", enum_name).unwrap();
        }
        write!(s, "}}").unwrap();
        s
    }
}