//! Unofficial Rust client for the `LabKey` Server REST API.
//!
//! This crate provides typed, async access to `LabKey`'s HTTP endpoints for query,
//! security, domain, experiment, and other modules. It is a port of the official
//! [`@labkey/api`](https://github.com/LabKey/labkey-api-js) JavaScript/TypeScript
//! client, adapted for idiomatic Rust.
//!
//! This crate is not affiliated with or endorsed by `LabKey` Corporation.
//!
//! # Example
//!
//! ```no_run
//! use labkey_rs::{ClientConfig, Credential, LabkeyClient};
//!
//! let config = ClientConfig::new(
//!     "https://labkey.example.com/labkey",
//!     Credential::Basic {
//!         email: "user@example.com".into(),
//!         password: "secret".into(),
//!     },
//!     "/MyProject/MyFolder",
//! );
//! let client = LabkeyClient::new(config).expect("valid configuration");
//! ```

pub mod client;
pub mod common;
pub mod domain;
pub mod error;
pub mod experiment;
pub mod filter;
pub mod query;
pub mod security;

pub use client::{ClientConfig, Credential, LabkeyClient};
pub use error::LabkeyError;
