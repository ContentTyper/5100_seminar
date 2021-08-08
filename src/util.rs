use std::{ io, fs };
use std::path::{ Path, PathBuf, Component };
use std::borrow::Cow;
use anyhow::Context;
use bstr::ByteSlice;
use encoding_rs::Encoding;
use flate2::bufread::DeflateDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;


pub enum Decoder<R: io::BufRead> {
    None(R),
    Deflate(DeflateDecoder<R>),
    Zstd(ZstdDecoder<'static, R>