
pub struct Eof;


#[inline]
pub fn take(input: &[u8], n: usize) -> Result<(&[u8], &[u8]), Eof> {
    if input.len() >= n {
        let (prefix, suffix) = input.split_at(n);
        Ok((suffix, prefix))