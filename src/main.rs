
mod util;

use std::{ cmp, env, fs };
use std::io::{ self, Read };
use std::path::{ Path, PathBuf };
use argh::FromArgs;
use anyhow::Context;
use bstr::ByteSlice;
use encoding_rs::Encoding;
use rayon::prelude::*;
use memmap2::MmapOptions;
use flate2::bufread::DeflateDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;
use zip_parser::{ compress, system, ZipArchive, CentralFileHeader };
use util::{
    Decoder, Crc32Checker, FilenameEncoding,
    dos2time, path_join, path_open, sanitize_setuid
};


/// unzrip - extract compressed files in a ZIP archive
#[derive(FromArgs)]
struct Options {
    /// path of the ZIP archive(s).
    #[argh(positional)]
    file: Vec<PathBuf>,

    /// an optional directory to which to extract files.
    #[argh(option, short = 'd')]
    exdir: Option<PathBuf>,

    /// specify character set used to decode filename,
    /// which will be automatically detected by default.
    #[argh(option, short = 'O')]
    charset: Option<String>,

    /// try to keep the original filename,
    /// which will ignore the charset.
    #[argh(switch)]
    keep_origin_filename: bool
}

fn main() -> anyhow::Result<()> {
    let options: Options = argh::from_env();

    let target_dir = if let Some(exdir) = options.exdir {
        exdir
    } else {
        env::current_dir()?
    };
    let encoding = if options.keep_origin_filename {
        FilenameEncoding::Os
    } else if let Some(label) = options.charset {
        let encoding = Encoding::for_label(label.as_bytes()).context("invalid encoding label")?;
        FilenameEncoding::Charset(encoding)
    } else {
        FilenameEncoding::Auto
    };

    for file in options.file.iter() {
        unzip(encoding, &target_dir, file)?;
    }

    Ok(())
}

fn unzip(encoding: FilenameEncoding, target_dir: &Path, path: &Path) -> anyhow::Result<()> {
    println!("Archive: {}", path.display());

    let fd = fs::File::open(path)?;
    let buf = unsafe {
        MmapOptions::new().map_copy_read_only(&fd)?
    };

    let zip = ZipArchive::parse(&buf)?;
    let len: usize = zip.eocdr().cd_entries.try_into()?;
    let len = cmp::min(len, 128);

    zip.entries()?
        .try_fold(Vec::with_capacity(len), |mut acc, e| e.map(|e| {
            acc.push(e);
            acc
        }))?
        .par_iter()
        .try_for_each(|cfh| do_entry(encoding, &zip, &cfh, target_dir))?;

    Ok(())
}

fn do_entry(