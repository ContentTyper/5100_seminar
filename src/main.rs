
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