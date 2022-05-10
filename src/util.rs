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
                let msg = format!("crc32 check failed. expect: {}, got: {}",
                    self.expect,
                    crc
                );
                return Err(io::Error::new(io::ErrorKind::InvalidData, msg))
            }
        } else {
            self.hasher.update(&buf[..n]);
        }

        Ok(n)
    }
}

#[derive(Clone, Copy)]
pub enum FilenameEncoding {
    Os,
    Charset(&'static Encoding),
    Auto
}

impl FilenameEncoding {
    pub fn decode<'a>(self, name: &'a [u8]) -> anyhow::Result<Cow<'a, Path>> {
        fn cow_str_to_path<'a>(name: Cow<'a, str>) -> Cow<'a, Path> {
            match name {
                Cow::Borrowed(name) => Cow::Borrowed(Path::new(name)),
                Cow::Owned(name) => Cow::Owned(name.into())
            }
        }

        match self {
            FilenameEncoding::Os => {
                name.to_path()
                    .map(Cow::Borrowed)
                    .context("Convert to os str failed")
                    .with_context(|| String::from_utf8_lossy(name).into_owned())
            },
            FilenameEncoding::Charset(encoding) => {
                let (name, ..) = encoding.decode(name);
                Ok(cow_str_to_path(name))
            },
            FilenameEncoding::Auto => if let Ok(name) = std::str::from_utf8(name) {
                Ok(Path::new(name).into())
            } else {
                let mut encoding_detector = chardetng::EncodingDetector::new();
                encoding_detector.feed(name, true);
                let (name, ..) = encoding_detector.guess(None, false).decode(name);
                Ok(cow_str_to_path(name))
            }
        }
    }
}

pub fn dos2time(dos_date: u16, dos_time: u16)
    -> anyhow::Result<time::PrimitiveDateTime>
{
    let sec = (dos_time & 0x1f) * 2;
    let min = (dos_time >> 5) & 0x3f;
    let hour = dos_time >> 11;

    let day = dos_date & 0x1f;
    let mon = (dos_date >> 5) & 0xf;
    let year = (dos_date >> 9) + 1980;

    let mon: u8 = mon.try_into().context("mon cast")?;
    let mon: time::Month = mon.try_into()?;

    let time = time::Time::from_hms(
        hour.try_into().context("hour cast")?,
        min.try_into().context("min cast")?,
        sec.try_into().context("sec cast")?
    )?;
    let date = time