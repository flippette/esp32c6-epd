use proc_macro2::TokenStream;
use quote::quote;
use std::{fs, mem, path::PathBuf, process::Command};
use syn::{Error, LitStr, Result};

pub fn expand(input: LitStr) -> Result<TokenStream> {
    let path = input
        .value()
        .parse::<PathBuf>()
        .map_err(|_| Error::new_spanned(&input, "invalid path!"))?;

    if !matches!(path.try_exists(), Ok(true)) {
        return Err(Error::new_spanned(&input, "file does not exist!"));
    }

    if !Command::new("ffmpeg")
        .arg("-version")
        .status()
        .is_ok_and(|status| status.success())
    {
        return Err(Error::new_spanned(&input, "failed to execute ffmpeg!"));
    }

    let tmp = tempfile::tempdir()
        .map_err(|_| Error::new_spanned(&input, "failed to create tempdir!"))?;

    if !Command::new("ffmpeg")
        .arg("-i")
        .arg(&path)
        .args(["-pix_fmt", "monob"])
        .args(["-vf", "negate"])
        .args(["-r", "1"])
        .arg(tmp.path().join("frame%04d.bmp"))
        .status()
        .is_ok_and(|status| status.success())
    {
        return Err(Error::new_spanned(&input, "ffmpeg transcode failed!"));
    }

    let includes = fs::read_dir(tmp.path())
        .map_err(|_| Error::new_spanned(&input, "failed to walk temp dir!"))?
        .map_while(|res| res.ok())
        .map(|entry| {
            let path = entry.path();
            let path = path.to_string_lossy();
            quote!(::core::include_bytes!(#path),)
        })
        .collect::<TokenStream>();

    mem::forget(tmp);

    Ok(quote!(&[#includes]))
}
