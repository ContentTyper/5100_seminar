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
    Zstd(ZstdDecoder<'static, R>)
}

impl<R: io::BufRead> io::Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Decoder::None(reader) => io::Read::read(reader, buf),
            Decoder::Deflate(reader) => io::Read::read(reader, buf),
            Decoder::Zstd(reader) => io::Read::read(reader, buf)
        }
    }
}

pub struct Crc32Checker<R> {
    reader: R,
    expect: u32,
    hasher: crc32fast::Hasher,
}

impl<R> Crc32Checker<R> {
    pub fn new(reader: R, expect: u32) -> Crc32Checker<R> {
        Crc32Checker {
            reader, expect,
            hasher: crc32fast::Hasher::new()
        }
    }
}

impl<R: io::Read> io::Read for Crc32Checker<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = io::Read::read(&mut self.reader, buf)?;

        if n == 0 {
            let crc = self.hasher.clone().finalize();
            if crc != self.expect {
     