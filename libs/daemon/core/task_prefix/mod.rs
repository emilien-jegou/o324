mod cache;
mod index;

// Expose the public-facing API.
//pub use builder::IndexBuilder;
pub use index::PrefixIndex;

#[cfg(test)]
mod tests;
