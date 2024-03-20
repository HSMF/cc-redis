use serde::{
    de::{self, MapAccess, SeqAccess},
    forward_to_deserialize_any, Deserialize,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("trailing characters at position {0}")]
    TrailingCharacters(usize),
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("syntax error at {0}")]
    Syntax(usize),
    #[error("provided negative length at {0}")]
    NegativeLength(usize),
    #[error("failed to parse int at {0}")]
    ParseIntError(usize),
    #[error("expected array at {0}")]
    ExpectedArray(usize),
    #[error("map has no associated value at {0}")]
    MissingValue(usize),
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

pub struct Deserializer<'de> {
    input: &'de [u8],
    orig_len: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input,
            orig_len: input.len(),
        }
    }

    fn position(&self) -> usize {
        self.orig_len - self.input.len()
    }

    /// advances the input by [tag] and returns true if the input starts with the tag,
    /// returns false otherwise
    #[must_use]
    fn tag(&mut self, tag: &[u8]) -> bool {
        if self.input.starts_with(tag) {
            self.input = &self.input[tag.len()..];
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Result<u8, Error> {
        self.input.first().copied().ok_or(Error::UnexpectedEof)
    }

    fn advance(&mut self) -> Result<u8, Error> {
        let first = self.input.first().copied().ok_or(Error::UnexpectedEof)?;
        self.input = &self.input[1..];
        Ok(first)
    }

    fn until_crlf(&mut self) -> Result<&'de [u8], Error> {
        let idx = self
            .input
            .windows(2)
            .enumerate()
            .find_map(|(i, win)| if win == b"\r\n" { Some(i) } else { None })
            .ok_or(Error::UnexpectedEof)?;
        let (buf, b) = self.input.split_at(idx);
        self.input = &b[2..];
        Ok(buf)
    }

    fn parse_int(&self, buf: &[u8], position: usize) -> Result<i64, Error> {
        atoi::atoi(buf).ok_or(Error::ParseIntError(position))
    }

    fn take(&mut self, n: usize) -> Result<&'de [u8], Error> {
        if self.input.len() < n {
            return Err(Error::UnexpectedEof);
        }

        let buf = &self.input[..n];
        self.input = &self.input[n..];
        Ok(buf)
    }
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters(deserializer.position()))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.advance()? {
            b'+' => {
                let buf = self.until_crlf()?;
                visitor.visit_borrowed_bytes(buf)
            }
            b'$' => {
                let pos = self.position();
                let len = self.until_crlf()?;
                let len = self.parse_int(len, pos)?;
                let len: usize = len.try_into().map_err(|_| Error::NegativeLength(pos))?;
                let buf = self.take(len)?;
                self.tag(b"\r\n")
                    .then_some(())
                    .ok_or(Error::Syntax(self.position()))?;
                visitor.visit_borrowed_bytes(buf)
            }
            b':' => {
                let pos = self.position();
                let int = self.until_crlf()?;
                let int = self.parse_int(int, pos)?;
                visitor.visit_i64(int)
            }
            b'#' => {
                let pos = self.position();
                let b = self.until_crlf()?;
                let b = match b {
                    [b't'] => true,
                    [b'f'] => false,
                    _ => return Err(Error::Syntax(pos)),
                };

                visitor.visit_bool(b)
            }

            _ => Err(Error::Syntax(self.position())),
        }
    }

    forward_to_deserialize_any! {bool i8 i16 i32 i64 u8 u16 u32 u64 bytes str string ignored_any}

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.advance()? != b'*' {
            return Err(Error::ExpectedArray(self.position()));
        }
        let pos = self.position();
        let len = self.until_crlf()?;
        let len = self.parse_int(len, pos)?;
        let len: usize = len.try_into().map_err(|_| Error::NegativeLength(pos))?;

        let value = visitor.visit_seq(Array::new(self, len))?;

        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.advance()? != b'%' {
            return Err(Error::ExpectedArray(self.position()));
        }
        let pos = self.position();
        let len = self.until_crlf()?;
        let len = self.parse_int(len, pos)?;
        let len: usize = len.try_into().map_err(|_| Error::NegativeLength(pos))?;

        let value = visitor.visit_map(Array::new(self, len))?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
}

struct Array<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'a, 'de: 'a> SeqAccess<'de> for Array<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }

        self.len -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'a, 'de: 'a> MapAccess<'de> for Array<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Err(Error::MissingValue(self.de.position()));
        }

        self.len -= 1;
        seed.deserialize(&mut *self.de)
    }
}

impl<'a, 'de: 'a> Array<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self { de, len }
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct Foo(i32);

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct Bar {
        a: i32,
        b: String,
        c: Foo,
    }

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    enum Baz {
        A,
    }

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    #[serde(untagged)]
    enum Untagged {
        Int(i64),
        String(String),
        Array(Vec<i32>),
    }

    fn prs<'a, T>(b: &'a [u8]) -> T
    where
        T: Deserialize<'a>,
    {
        from_bytes(b).expect("succeeded with parse")
    }

    trait Str {
        fn to_bytes(self) -> Vec<u8>;
    }

    impl Str for &str {
        fn to_bytes(self) -> Vec<u8> {
            let mut o = String::new();
            use std::fmt::Write;

            write!(o, "{self}\r\n").unwrap();

            o.into_bytes()
        }
    }

    impl Str for &[&str] {
        fn to_bytes(self) -> Vec<u8> {
            let mut o = String::new();
            use std::fmt::Write;

            for i in self {
                write!(o, "{i}\r\n").unwrap();
            }

            o.into_bytes()
        }
    }

    impl Str for &[u8] {
        fn to_bytes(self) -> Vec<u8> {
            let mut o = Vec::new();
            o.extend_from_slice(self);
            o.extend_from_slice(b"\r\n");
            o
        }
    }

    impl Str for &[&[u8]] {
        fn to_bytes(self) -> Vec<u8> {
            let mut o = Vec::new();

            for i in self {
                o.extend_from_slice(i);
                o.extend_from_slice(b"\r\n");
            }

            o
        }
    }

    macro_rules! case {
        ($ty:ty,$name:ident, $s:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(prs::<$ty>(&($s).to_bytes()), $expected);
            }
        };
    }

    case!(i32, de_i32, ":1", 1);
    case!(i64, de_i64, ":15", 15);
    case!(bool, de_bool, "#t", true);
    case!(&[u8], de_bytes, "+abc", "abc".as_bytes());
    case!(&str, de_str, "+abc", "abc");
    case!(&str, de_bulk_str, "$5\r\nhello", "hello");
    case!(&str, de_bulk_str_nl, "$7\r\nhel\r\nlo", "hel\r\nlo");
    case!(Vec<i32>, int_array_empty, "*0", []);
    case!(
        Vec<i32>,
        int_array_some,
        ["*3", ":1", ":2", ":3"],
        [1, 2, 3]
    );
    case!(
        (i32, String, bool),
        tuple,
        ["*3", ":1", "+hello", "#t"],
        (1, "hello".into(), true)
    );

    case!(Foo, newtype, ":1", Foo(1));
    case!(
        Bar,
        struct_case,
        ["%3", "+a", ":17", "+c", ":-1", "+b", "+hello"],
        Bar {
            a: 17,
            c: Foo(-1),
            b: "hello".into()
        }
    );

    // case!(Baz, enum_case, (todo!() as &str), Baz::A);
    case!(Untagged, untagged_int, ":1", Untagged::Int(1));
    case!(
        Untagged,
        untagged_string,
        "+hey",
        Untagged::String("hey".into())
    );
    case!(
        Untagged,
        untagged_array,
        ["*3", ":1", ":2", ":3"],
        Untagged::Array([1, 2, 3].into())
    );

    case!(Option<String>, option_null_string, "$-1", None);
    case!(Option<i32>, option_int, "_", None);
    case!(Option<Vec<i32>>, option_null_array, "*-1", None);
}
