pub use patronus_macros::patronus;

pub enum Setter<T> {
    Unset,
    Set(T),
}

impl<T> Default for Setter<T> {
    fn default() -> Setter<T> {
        Setter::Unset
    }
}

impl<T> Setter<T> {
    pub fn to_option(self) -> Option<T> {
        match self {
            Self::Set(e) => Some(e),
            Self::Unset => None,
        }
    }

    pub fn or(self, value: T) -> T {
        match self {
            Self::Set(x) => x,
            Self::Unset => value,
        }
    }
}
