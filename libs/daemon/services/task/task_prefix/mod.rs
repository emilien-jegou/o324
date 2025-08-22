mod index;

// Expose the public-facing API.
//pub use builder::IndexBuilder;
pub use index::TaskPrefixRepository;

#[cfg(test)]
mod tests;
