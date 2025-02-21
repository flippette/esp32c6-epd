//! Helper macros.

// TODO: this should ultimately be a derive macro.
/// Define an error enum in the current scope.
#[macro_export]
macro_rules! error_def {
    ($($variant:ident => $inner:ty = $msg:literal),+ $(,)?) => {
        pub enum Error {
            $($variant($inner)),+
        }

        $(
            impl ::core::convert::From<$inner> for Error {
                fn from(inner: $inner) -> Self {
                    Self::$variant(inner)
                }
            }
        )+

        impl ::defmt::Format for Error {
            fn format(&self, fmt: ::defmt::Formatter<'_>) {
                match self {
                    $(Self::$variant(inner) =>
                        ::defmt::write!(fmt, $msg, inner)),+
                }
            }
        }
    };
}

/// Convenient macro for creating a static cell in place.
#[macro_export]
macro_rules! make_static {
    // Runtime-initialized in place
    ($(#[$m:meta])* $type:ty = $val:expr) => {{
        $(#[$m])*
        static __CELL: ::static_cell::StaticCell<$type> =
            ::static_cell::StaticCell::new();
        __CELL.uninit().write($val)
    }};

    // Compile-time-initialized
    ($(#[$m:meta])* const $type:ty = $val:expr) => {{
        $(#[$m])*
        static __CELL: ::static_cell::ConstStaticCell<$type> =
            ::static_cell::ConstStaticCell::new($val);
        __CELL.take()
    }};
}
