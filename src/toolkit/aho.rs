#[macro_export]
macro_rules! define_aho_matcher {
    ($v: vis $name: ident for: $($pattern: literal),+ $(,)?) => {
        $v static $name: std::sync::LazyLock<aho_corasick::AhoCorasick> = std::sync::LazyLock::new(
            || {
                const PATTERN: &'static [&'static str] = &[$($pattern,)+];
                aho_corasick::AhoCorasickBuilder::new().build(PATTERN).unwrap()
            }
        );
    };
    ($v: vis $name: ident as ascii_case_insensitive for: $($pattern: literal),+ $(,)?) => {
        $v static $name: std::sync::LazyLock<aho_corasick::AhoCorasick> = std::sync::LazyLock::new(
            || {
                const PATTERN: &'static [&'static str] = &[$($pattern,)+];
                aho_corasick::AhoCorasickBuilder::new().ascii_case_insensitive(true).build(PATTERN).unwrap()
            }
        );
    };
    ($v: vis $name: ident for $b: block: $($pattern: literal),+ $(,)?) => {
        $v static $name: std::sync::LazyLock<aho_corasick::AhoCorasick> = std::sync::LazyLock::new(
            || {
                const PATTERN: &'static [&'static str] = &[$($pattern,)+];
                let mut $name = aho_corasick::AhoCorasickBuilder::new();
                $b;
                $name
            }
        );
    };
}