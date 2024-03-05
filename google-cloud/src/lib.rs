#![warn(missing_docs)]
//! Asynchronous Rust bindings for Google Cloud Platform gRPC APIs.

#[cfg(feature = "google-cloud-derive")]
extern crate google_cloud_derive;

/// Authorization/authentication related utilities.
pub mod authorize;
/// Error handling utilities.
pub mod error;

/// Cloud Storage bindings.
#[cfg(feature = "storage")]
pub mod storage;

#[cfg(test)]
mod tests;
