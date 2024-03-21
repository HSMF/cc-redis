#[derive(Debug)]
pub struct CaseInsensitive<S>(pub S);

impl<S, T> PartialEq<CaseInsensitive<T>> for CaseInsensitive<S>
where
    S: AsRef<str>,
    T: AsRef<str>,
{
    fn eq(&self, other: &CaseInsensitive<T>) -> bool {
        let me = self.0.as_ref();
        let you = other.0.as_ref();
        me.eq_ignore_ascii_case(you)
    }
}

impl<S> PartialEq<&str> for CaseInsensitive<S>
where
    S: AsRef<str>,
{
    fn eq(&self, other: &&str) -> bool {
        let me = self.0.as_ref();
        let you = other;
        me.eq_ignore_ascii_case(you)
    }
}

impl<T> PartialEq<CaseInsensitive<T>> for str
where
    T: AsRef<str>,
{
    fn eq(&self, other: &CaseInsensitive<T>) -> bool {
        let you = other.0.as_ref();
        self.eq_ignore_ascii_case(you)
    }
}


impl<S> Eq for CaseInsensitive<S> where S: AsRef<str> {}
