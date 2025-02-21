mod expand;

use proc_macro::TokenStream;
use syn::{LitStr, parse_macro_input};

/// Given the path of a video file, run it through `ffmpeg` and return frame
/// data as a slice of QOI image files.
///
/// This slice is sorted in reverse (that is, from the last frame to the first),
/// for... reasons.
#[proc_macro]
pub fn embed(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as LitStr);
    expand::expand(path)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
