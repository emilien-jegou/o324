pub use patronus_macros::patronus;

#[derive(Default, PartialEq, Debug)]
pub enum Setter<T> {
    #[default]
    Unset,
    Set(T),
}

impl<T> From<Setter<T>> for Option<T> {
    fn from(val: Setter<T>) -> Self {
        match val {
            Setter::Set(e) => Some(e),
            Setter::Unset => None,
        }
    }
}

impl<T> From<Option<T>> for Setter<T> {
    fn from(val: Option<T>) -> Self {
        match val {
            Option::Some(e) => Setter::Set(e),
            Option::None => Setter::Unset,
        }
    }
}

impl<T> Setter<T> {
    pub fn or(self, value: T) -> T {
        match self {
            Self::Set(x) => x,
            Self::Unset => value,
        }
    }
}
