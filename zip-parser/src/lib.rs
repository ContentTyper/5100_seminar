
//! https://www.hanshq.net/zip.html#zip

mod util;

use thiserror::Error;
use memchr::memmem::rfind;
use util::{ Eof, take, read_u16, read_u32 };


pub mod compress {
    pub const STORE: u16   = 0;
    pub const DEFLATE: u16 = 8;
    pub const ZSTD: u16    = 93;
}

pub mod system {
    pub const DOS: u16 = 0;
    pub const UNIX: u16 = 3;
}

#[non_exhaustive]
#[derive(Debug)]
pub struct EocdRecord<'a> {
    pub disk_nbr: u16,
    pub cd_start_disk: u16,
    pub disk_cd_entries: u16,
    pub cd_entries: u16,
    pub cd_size: u32,
    pub cd_offset: u32,
    pub comment: &'a [u8]
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("eof")]
    Eof,
    #[error("bad eocdr magic number")]
    BadEocdr,
    #[error("bad cfh magic number")]
    BadCfh,
    #[error("bad lfh magic number")]
    BadLfh,
    #[error("not supported")]
    Unsupported,
    #[error("offset overflow")]
    OffsetOverflow
}

impl From<Eof> for Error {
    #[inline]
    fn from(_err: Eof) -> Error {
        Error::Eof
    }
}

impl EocdRecord<'_> {
    fn find(buf: &[u8]) -> Result<EocdRecord<'_>, Error> {
        const EOCDR_SIGNATURE: &[u8; 4] = &[b'P', b'K', 5, 6];
        const MAX_BACK_OFFSET: usize = 1024 * 128;

        let eocdr_buf = {
            let max_back_buf = buf.len()
                .checked_sub(MAX_BACK_OFFSET)
                .map(|pos| &buf[pos..])
                .unwrap_or(buf);

            let eocdr_offset = rfind(max_back_buf, EOCDR_SIGNATURE)
                .ok_or(Error::BadEocdr)?;
            &max_back_buf[eocdr_offset..]
        };

        let input = eocdr_buf;
        let (input, _) = take(input, EOCDR_SIGNATURE.len())?;
        let (input, disk_nbr) = read_u16(input)?;