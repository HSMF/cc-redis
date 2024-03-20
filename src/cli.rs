use std::{ffi::CStr, io::Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Bar {
    hello: String,
    baz: i32,
}

fn foo<T>(x: T) -> anyhow::Result<()>
where
    T: Serialize,
{
    let wireable = redis::serializer::to_bytes(&x)?;
    std::io::stdout().write_all(&wireable)?;
    Ok(())
}

fn prs<'a, T>(b: &'a [u8]) -> anyhow::Result<T>
where
    T: Deserialize<'a>,
{
    let v = redis::deserializer::from_bytes(b)?;
    Ok(v)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    foo("hello world")?;
    let x = CStr::from_bytes_with_nul(b"a\0")?;
    foo(x)?;
    foo(1)?;
    foo([1, 2].as_slice())?;
    foo([1, 2])?;
    foo(vec![3, 4])?;
    foo(-f64::NAN)?;

    foo(Bar {
        baz: 1,
        hello: "world".to_owned(),
    })?;

    println!("==================");

    dbg!(prs::<i32>(b":1\r\n")?);
    dbg!(prs::<&[u8]>(b"+ping\r\n")?);

    Ok(())
}
