pub use patronus_macros::patronus;

#[derive(Default, PartialEq, Debug)]
pub enum Setter<T> {
    #[default]
    Unset,
    Set(T),
}

impl<T> Into<Option<T>> for Setter<T> {
    fn into(self) -> Option<T> {
        match self {
            Self::Set(e) => Some(e),
            Self::Unset => None,
        }
    }
}

impl<T> Into<Setter<T>> for Option<T> {
    fn into(self) -> Setter<T> {
        match self {
            Self::Some(e) => Setter::Set(e),
            Self::None => Setter::Unset,
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
