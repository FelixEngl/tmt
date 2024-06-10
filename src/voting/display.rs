use std::fmt::{Display, Write};

macro_rules! impl_display_for_displaytree {
    ($($target: ident),+) => {
        $(
            impl Display for $target {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    let mut code_formatter = $crate::voting::display::IndentWriter::new(f);
                    $crate::voting::display::DisplayTree::fmt(self, &mut code_formatter)
                }
            }
        )+
    };
}
pub(crate) use impl_display_for_displaytree;

/// Allows to display atree
pub trait DisplayTree: Display {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result;
}

pub struct IndentWriter<'a, T: Write> {
    f: &'a mut T,
    level: usize,
    indent: String
}

impl<'a, T> IndentWriter<'a, T> where T: Write {
    pub fn new(f: &'a mut T) -> Self {
        Self {
            f,
            level: 0,
            indent: String::new()
        }
    }

    pub fn indent(&mut self, value: usize) {
        self.level = self.level.saturating_add(value);
        self.indent = " ".repeat(self.level);
    }

    pub fn dedent(&mut self, value: usize) {
        self.level = self.level.saturating_sub(value);
        self.indent = " ".repeat(self.level);
    }
}

impl<T> Write for IndentWriter<'_, T> where T: Write {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if s.ends_with("\n") {
            write!(self.f, "{}{}", s, self.indent)
        } else {
            write!(self.f, "{}", s)
        }

    }
}