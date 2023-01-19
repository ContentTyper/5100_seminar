
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
        let (input, cd_start_disk) = read_u16(input)?;
        let (input, disk_cd_entries) = read_u16(input)?;
        let (input, cd_entries) = read_u16(input)?;
        let (input, cd_size) = read_u32(input)?;
        let (input, cd_offset) = read_u32(input)?;
        let (input, comment_len) = read_u16(input)?;
        let (_input, comment) = take(input, comment_len.into())?;

        Ok(EocdRecord {
            disk_nbr,
            cd_start_disk,
            disk_cd_entries,
            cd_entries,
            cd_size,
            cd_offset,
            comment
        })
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct CentralFileHeader<'a> {
    pub made_by_ver: u16,
    pub extract_ver: u16,
    pub gp_flag: u16,
    pub method: u16,
    pub mod_time: u16,
    pub mod_date: u16,
    pub crc32: u32,
    pub comp_size: u32,
    pub uncomp_size: u32,
    pub disk_nbr_start: u16,
    pub int_attrs: u16,
    pub ext_attrs: u32,
    pub lfh_offset: u32,
    pub name: &'a [u8],
    pub extra: &'a [u8],
    pub comment: &'a [u8]
}

impl CentralFileHeader<'_> {
    fn parse(input: &[u8]) -> Result<(&[u8], CentralFileHeader<'_>), Error> {
        const CFH_SIGNATURE: &[u8; 4] = &[b'P', b'K', 1, 2];

        let (input, expect_sig) = take(input, CFH_SIGNATURE.len())?;
        if expect_sig != CFH_SIGNATURE {
            return Err(Error::BadCfh);
        }

        let (input, made_by_ver) = read_u16(input)?;
        let (input, extract_ver) = read_u16(input)?;
        let (input, gp_flag) = read_u16(input)?;
        let (input, method) = read_u16(input)?;
        let (input, mod_time) = read_u16(input)?;
        let (input, mod_date) = read_u16(input)?;
        let (input, crc32) = read_u32(input)?;
        let (input, comp_size) = read_u32(input)?;
        let (input, uncomp_size) = read_u32(input)?;
        let (input, name_len) = read_u16(input)?;
        let (input, extra_len) = read_u16(input)?;
        let (input, comment_len) = read_u16(input)?;
        let (input, disk_nbr_start) = read_u16(input)?;
        let (input, int_attrs) = read_u16(input)?;
        let (input, ext_attrs) = read_u32(input)?;
        let (input, lfh_offset) = read_u32(input)?;
        let (input, name) = take(input, name_len.into())?;
        let (input, extra) = take(input, extra_len.into())?;
        let (input, comment) = take(input, comment_len.into())?;

        let header = CentralFileHeader {
            made_by_ver,
            extract_ver,
            gp_flag,
            method,
            mod_time,
            mod_date,
            crc32,
            comp_size,
            uncomp_size,
            disk_nbr_start,
            int_attrs,
            ext_attrs,
            lfh_offset,
            name,
            extra,
            comment
        };

        Ok((input, header))
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct LocalFileHeader<'a> {
    pub extract_ver: u16,
    pub gp_flag: u16,
    pub method: u16,
    pub mod_time: u16,
    pub mod_date: u16,
    pub crc32: u32,
    pub comp_size: u32,
    pub uncomp_size: u32,
    pub name: &'a [u8],
    pub extra: &'a [u8]
}

impl LocalFileHeader<'_> {
    fn parse(input: &[u8]) -> Result<(&[u8], LocalFileHeader<'_>), Error> {
        const LFH_SIGNATURE: &[u8; 4] = &[b'P', b'K', 3, 4];

        let (input, expect_sig) = take(input, LFH_SIGNATURE.len())?;
        if expect_sig != LFH_SIGNATURE {
            return Err(Error::BadLfh);
        }

        let (input, extract_ver) = read_u16(input)?;
        let (input, gp_flag) = read_u16(input)?;
        let (input, method) = read_u16(input)?;
        let (input, mod_time) = read_u16(input)?;
        let (input, mod_date) = read_u16(input)?;
        let (input, crc32) = read_u32(input)?;
        let (input, comp_size) = read_u32(input)?;
        let (input, uncomp_size) = read_u32(input)?;
        let (input, name_len) = read_u16(input)?;
        let (input, extra_len) = read_u16(input)?;
        let (input, name) = take(input, name_len.into())?;
        let (input, extra) = take(input, extra_len.into())?;

        let header = LocalFileHeader {
            extract_ver,
            gp_flag,
            method,
            mod_time,
            mod_date,
            crc32,
            comp_size,
            uncomp_size,
            name,
            extra
        };

        Ok((input, header))
    }