#[doc(hidden)]
#[macro_export]
macro_rules! text_impl {
    // c:"text"
    ([$style_id:ident : $lit:literal $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment {
            style: Some($crate::Style::$style_id), text: $lit, ticks: true
        })
    };
    // c:{expr}
    ([$style_id:ident : {$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment {
            style: Some($crate::Style::$style_id), text: $ex, ticks: true
        })
    };
    // c!"text"
    ([$style_id:ident ! $lit:literal $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment {
            style: Some($crate::Style::$style_id), text: $lit, ticks: false
        })
    };
    // c!{expr}
    ([$style_id:ident ! {$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment {
            style: Some($crate::Style::$style_id), text: $ex, ticks: false
        })
    };
    // "text"
    ([$lit:literal $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment::new($lit))
    };
    // {expr}
    ([{$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment::new($ex))
    };
    ([], $($done:tt)*) => {
        &[$($done)*]
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! rhs_impl {
    // {expr}
    ({ $($inner:tt)* }) => {
        $crate::sections!( $($inner)* )
    };
    ($e:expr) => {
        $e
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sections_impl {
    // ["text"]
    (@[ [$($text:tt)*] $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::Text($crate::text![ $($text)* ])
        )
    };
    // table {
    //     "foo" => {...}
    //     "bar" => {...}
    // }
    (@[ table $mode:ident { $( $key:literal => $rhs:tt )* } $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::Table($crate::TableMode::$mode, &[$(
                ( $key, $crate::rhs_impl!($rhs) ),
            )*])
        )
    };
    // "NAME" {...}
    (@[ $name:literal $rhs:tt $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::Name($name, $crate::rhs_impl!($rhs))
        )
    };
    // Short ["text"]
    (@[ $wrapper:ident [$($text:tt)* ] $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::$wrapper(
                &$crate::HelpSection::Text($crate::text![ $($text)* ])
            )
        )
    };
    // Short table {
    //     "foo" => {...}
    //     "bar" => {...}
    // }
    (@[ $wrapper:ident table $mode:ident { $( $key:literal => $rhs:tt )* } $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::$wrapper(
                &$crate::HelpSection::Table($crate::TableMode::$mode, &[$(
                    ( $key, $crate::rhs_impl!($rhs) ),
                )*])
            )
        )
    };
    // Short "NAME" {...}
    (@[ $wrapper:ident $name:literal $rhs:tt $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[$($rest)*] $($done)*,
            $crate::HelpSection::$wrapper(
                &$crate::HelpSection::Name($name, $crate::rhs_impl!($rhs))
            )
        )
    };
    // {expr}
    (@[ { $($inner:tt)* } $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[$($rest)*] $($done)*,
            $($inner)*
        )
    };

    (@[], $($done:tt)*) => {
        &[ $($done)* ]
    };
}

/// Macro to declare a list of text segments. A segment can be written as
///
/// - `"text"` (a string literal)
/// - `{expr}` (an expression that evaluates to a string slice)
///
/// Each segment can be preceded by one of
///
/// - `c:`, where `c` is a [`Style`](crate::Style) variant; the segment is
///   styled if supported, otherwise it is wrapped in backticks
/// - `c!`, where `c` is a [`Style`](crate::Style) variant; the segment is
///   styled if supported, otherwise no formatting is applied
///
/// Each color style can be abbreviated with its first letter (cyan ➔ c, green ➔ g,
/// magenta ➔ m, red ➔ r, yellow ➔ y); use an uppercase letter to make it
/// bold (bold cyan ➔ C, etc.). The `Underline` and `UnderlineBold` styles
/// are abbreviated as `u` and `U`, respectively.
///
/// Segments are _not_ separated with commas, for example:
///
/// ```
/// # use helptext::text;
/// // "warning" is yellow and bold, "world" is cyan, or wrapped in backticks
/// let _segments = text!(Y!"warning" ": hello" c:"world");
///
/// // the value of an build-time environment variable is printed in magenta
/// let _segments = text!("version is " m!{env!("CARGO_PKG_VERSION")});
/// ```
#[macro_export]
macro_rules! text {
    () => {
        &[]
    };
    ($($rest:tt)*) => {
        $crate::text_impl!([ $($rest)* ])
    };
}

/// Macro to declare a list of help sections. This can be passed to
/// [`Help`](crate::Help) to print it.
///
/// There are three kinds of sections:
///
/// 1. Normal sections, wrapped in square brackets. Refer to the
///    [`text` macro][text] for the syntax. Example:
///
///    ```
///    # helptext::sections!(
///    ["test" c:"cyan" R!"bold red"]
///    # );
///    ```
///
///    Each section is terminated by a line break.
///
/// 2. Named sections. Example:
///
///    ```
///    # helptext::sections!(
///    "Usage" {
///        ["section 1"]
///        ["section 2"]
///    }
///    # );
///    ```
///
///    Named sections are always preceded by a blank line. Child sections are
///    indented with 4 spaces.
///
/// 3. Tables. Example:
///
///    ```
///    # helptext::sections!(
///    table Auto {
///        "argument 1" => {
///            ["help for argument 1"]
///        }
///        "argument 2" => {
///            ["help for argument 2"]
///            ["and some more help!"]
///        }
///    }
///    # );
///    ```
///
///    With short help, this is rendered as
///
///    ```text
///    argument 1   help for argument 1
///    argument 2   help for argument 2
///                 and some more help!
///    ```
///
///    With long help, this is rendered as
///
///    ```text
///    argument 1
///            help for argument 1
///
///    argument 2
///            help for argument 2
///            and some more help!
///    ```
///
///    The argument name (left column) must be a string literal. It is displayed
///    in color.
///
///    The `table` keyword must be followed by either `Auto` or `Compact`. If
///    `Compact` is used, then the compact format is used for both the short and
///    long help. If `Auto` is used, the compact format is used for short help
///    and the longer format is used for long help.
///
/// Beyond that, each section can be preceded by `Short` or `Long`. By default,
/// sections are included both in the long and short help. With the `Short`
/// modifier, it is _only_ shown in the short help, and sections preceded by
/// `Long` only appear in the long help. Example:
///
/// ```
/// # use helptext::sections;
/// sections!(
///     Short ["Short help text"]
///     Long ["This is more detailed help text"]
///     ["This is shown either way"]
///
///     table Auto {
///         "argument 1" => {
///             ["description"]
///             Long ["Further details only shown in long help"]
///         }
///         "argument 2" => {
///             Long ["This argument isn't shown in the short help"]
///         }
///     }
///
///     // table only shown in long help:
///     Long table Compact {}
///
///     Long "MORE DETAILS" {
///         ["named section only shown in long help"]
///     }
/// );
/// ```
///
/// You can factor parts of the help message into variables, and include them in the `sections!` macro:
///
/// - A single section is included by wrapping it in `{}`:
///
///   ```
///   # use helptext::{sections, Help, HelpSection};
///   const FOO: HelpSection = HelpSection::Text(sections![]);
///  
///   Help(sections![
///       ["Included section:"]
///       {FOO}
///   ]);
///   ```
///
/// - Content of named sections and table rows is included by replacing the `{...}` with the variable name:
///
///   ```
///   # use helptext::{sections, Help, HelpSection};
///   const FOO: &[HelpSection] = sections![
///       ["First line"]
///       ["Second line"]
///   ];
///  
///   Help(sections![
///       "Named section" FOO
///
///       table Auto {
///           "Foo" => FOO
///       }
///   ]);
///   ```
#[macro_export]
macro_rules! sections {
    () => {
        &[]
    };
    ($($rest:tt)*) => {
        $crate::sections_impl!(@[ $($rest)* ])
    };
}
