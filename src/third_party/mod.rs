/*! Implementations of [`TreeData`](crate::TreeData) for other crates.
 *
 * As `impl TreeData for Something` doesn't need to be public, they don't show up in rustdoc.
 * See the Cargo.toml for available features.
 */

pub mod common;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "messagepack")]
pub mod messagepack;
