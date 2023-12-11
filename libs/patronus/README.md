# Patronus

## Introduction

The patronus macro is a Rust procedural macro designed to streamline the
process of creating update structures similar.

The name of this library is inspired by the french word "patron" which can be
translate to the english word "Model".

## Example

```rust
#[patronus(UserUpdate)]
pub struct User {
    pub id: String,
    pub name: String,
    pub age: u8,
    pub email: Option<String>,
}


// patronus macro will expand to:
#[derive(Default)]
pub struct UserUpdate {
    pub id: Setter<String>,
    pub name: Setter<String>,
    pub age: Setter<u8>,
    pub email: Setter<Option<String>>,
}

impl UserUpdate {
    pub fn set_id(mut self, value: impl Into<String>) -> Self {
        self.id = Setter::Set(value.into());
        self
    }

    pub fn reset_id(mut self) -> Self {
        self.id = Setter::Unset;
        self
    }

    // ... Additional setter and resetter methods for each field
}

// ... then a few way to use it:

let user = UserUpdate::default()
    .set_id("123")
    .set_age(42)
    .set_email("my@email.com");


let mut user_1 = UserUpdate {
    id: Setter::Set("456"),
    age: Setter::Set(42),
    ..Default::default()
    // ... All others fields will be unset
};

```

## Credits

Inspiration was taken from sea-orm ActiveModel derive implementation and the
rust derive_builder crate.
