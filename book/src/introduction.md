# Introduction

labkey-rs is an unofficial Rust client for the [LabKey Server](https://www.labkey.org/) REST API. It provides typed, async access to LabKey's HTTP endpoints for querying data, managing security, working with assays and experiments, and more. It is a port of the official [`@labkey/api`](https://github.com/LabKey/labkey-api-js) JavaScript/TypeScript client, supplemented by the [Java client](https://github.com/LabKey/labkey-api-java) for endpoint coverage.

This book is a companion to the [API reference on docs.rs](https://docs.rs/labkey-rs). The API reference documents every type and method; this book explains how to use them together to get things done.

## Who this is for

You're using a LabKey Server instance and want to interact with it from Rust. Maybe you're building a data pipeline, a CLI tool, or integrating LabKey data into a larger application. You know some Rust and some LabKey, and you want to get productive quickly.

If you're new to LabKey itself, the [How LabKey Works](./guides/how-labkey-works.md) guide gives a brief orientation to the concepts that matter for this client, with links to LabKey's own documentation for anything deeper.

## How this book is organized

The **Guides** section walks through the core concepts in a logical order: setting up a client, understanding LabKey's data model, querying and modifying data, filtering and sorting, and handling errors. If you're new to this crate, start with [Getting Started](./guides/getting-started.md).

The **Recipes** section contains focused, self-contained solutions to common tasks like paginating through large result sets, importing data in bulk, or working with assays. Each recipe assumes you've read the getting-started guide but can otherwise be read independently.

The **Reference** section provides a [Module Map](./reference/module-map.md) that shows how the crate's modules correspond to LabKey's API controllers, so you can quickly find the right method for what you're trying to do.

## Other resources

The [API reference on docs.rs](https://docs.rs/labkey-rs) has comprehensive documentation for every public type, method, and module. The [examples directory](https://github.com/nrminor/labkey-rs/tree/main/examples) in the repository contains runnable programs that demonstrate common workflows. And the [LabKey documentation](https://www.labkey.org/Documentation/wiki-page.view?name=docs) is the authoritative source on how the server itself works.
