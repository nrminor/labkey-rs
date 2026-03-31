# Introduction

labkey-rs is an unofficial Rust client for the [LabKey Server](https://www.labkey.org/) REST API. It provides typed, async access to LabKey's HTTP endpoints for querying data, managing security, working with assays and experiments, and more. It is a port of the official [`@labkey/api`](https://github.com/LabKey/labkey-api-js) JavaScript/TypeScript client, supplemented by the [Java client](https://github.com/LabKey/labkey-api-java) for endpoint coverage.

> This is a third-party, community-maintained client. It is not an official LabKey product and is not supported by LabKey Corporation.

This book is a companion to the [API reference on docs.rs](https://docs.rs/labkey-rs). The API reference documents every type and method; this book explains how to use them together to get things done.

## Who this is for

You're using a LabKey Server instance and want to interact with it from Rust. Maybe you're building a data pipeline, a CLI tool, or integrating LabKey data into a larger application. You know some Rust and some LabKey, and you want to get productive quickly.

If you're new to LabKey itself, the [How LabKey Works](./guides/how-labkey-works.md) guide gives a brief orientation to the concepts that matter for this client, with links to LabKey's own documentation for anything deeper.

## How this book is organized

The **Guides** section walks through the core concepts in a logical order: setting up a client, understanding LabKey's data model, querying and modifying data, filtering and sorting, and handling errors. If you're new to this crate, start with [Getting Started](./guides/getting-started.md).

The **Recipes** section contains focused, self-contained solutions to common tasks like paginating through large result sets, importing data in bulk, or working with assays. Each recipe assumes you've read the getting-started guide but can otherwise be read independently.

For detailed API documentation — every public type, method, and field — see the [API reference on docs.rs](https://docs.rs/labkey-rs).

## Controller-to-module map

If you already know which LabKey controller you need, this table shows where to find it in the crate. Each crate module corresponds to one or more LabKey server controllers:

| Crate module | LabKey controller(s) | What it covers |
|---|---|---|
| `query` | `query` | Select, insert, update, delete, truncate, import, SQL, schema introspection |
| `filter` | — | Filter types and encoding (used by `query` methods) |
| `sort` | — | Sort specifications (used by `query` methods) |
| `security` | `security`, `core`, `project`, `admin`, `login`, `user` | Containers, groups, users, permissions, policies, sessions |
| `domain` | `property` | Domain and field metadata, create/update/delete domains |
| `list` | *(delegates to `domain`)* | List creation convenience wrapper |
| `experiment` | `assay` | Experiment runs, batches, materials, lineage |
| `assay` | `assay` | Assay designs, protocols, NAb, import runs |
| `di` | `dataintegration` | ETL transform configuration and execution |
| `pipeline` | — | Pipeline job status and analysis |
| `report` | `reports` | Report execution, sessions, data views |
| `message` | `announcements` | Send messages / notifications |
| `participant_group` | `participant-group` | Study participant group management |
| `specimen` | `specimen-api` | Specimen repository queries |
| `storage` | `storage` | Sample storage locations |
| `visualization` | `visualization` | Visualization saved reports |

The `client`, `error`, and `common` modules provide shared infrastructure (the `LabkeyClient` struct, error types, and common enums) rather than wrapping a specific controller.

## Other resources

The [examples directory](https://github.com/nrminor/labkey-rs/tree/main/examples) in the repository contains runnable programs that demonstrate common workflows. And the [LabKey documentation](https://www.labkey.org/Documentation/wiki-page.view?name=docs) is the authoritative source on how the server itself works.
