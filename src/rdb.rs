use nom::{
    bytes::complete::{tag, take},
    IResult,
};

pub struct Rdb {}

fn header(s: &[u8]) -> IResult<&[u8], ()> {
    let (s, _) = tag(b"REDIS")(s)?;
    Ok((s, ()))
}


fn version(s: &[u8]) -> IResult<&[u8], u32> {
    let (s, _) = take(4u32)(s)?;
    // let vers = atoi::atoi(vers).ok_or()?;
    Ok((s, 3))
}

impl Rdb {
    pub fn from_file(reader: &[u8]) -> Self {
        todo!();
    }
}
