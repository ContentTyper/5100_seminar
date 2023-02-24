
pub struct Eof;


#[inline]
pub fn take(input: &[u8], n: usize) -> Result<(&[u8], &[u8]), Eof> {
    if input.len() >= n {
        let (prefix, suffix) = input.split_at(n);
        Ok((suffix, prefix))
    } else {
        Err(Eof)
    }
}

#[inline]
pub fn read_u16(input: &[u8]) -> Result<(&[u8], u16), Eof> {
    let mut buf = [0; 2];
    let (input, output) = take(input, buf.len())?;
    buf.copy_from_slice(output);