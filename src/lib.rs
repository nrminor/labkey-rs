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
//! let client = LabkeyClient::new(ClientConfig {
//!     base_url: "https://labkey.example.com/labkey".into(),
//!     credential: Credential::Basic {
//!         email: "user@example.com".into(),
//!         password: "secret".into(),
//!     },
//!     container_path: "/MyProject/MyFolder".into(),
//! }).expect("valid configuration");
//! ```

pub mod client;
pub mod error;
pub mod filter;

pub use client::{ClientConfig, Credential, LabkeyClient};
pub use error::LabkeyError;
