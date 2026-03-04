# Feasibility Assessment: Unofficial Rust LabKey API Client

This document summarizes a thorough review of the `@labkey/api` JavaScript client (v1.48.0, Apache-2.0) and evaluates the feasibility of creating an unofficial Rust API client with complete feature parity, published on crates.io.

## What the JavaScript Client Is

The `@labkey/api` package is a browser-oriented TypeScript client for LabKey Server. It's structured as a collection of modules that each correspond to a LabKey Server controller, with each module's functions constructing HTTP requests to specific `.api` endpoints. The pattern is remarkably uniform across the entire codebase.

## Architecture of the JavaScript Client

The client has three layers.

**Layer 1 — Transport and URL construction.** `Ajax.ts` wraps `XMLHttpRequest` with CSRF token handling, content-type negotiation, and callback dispatch. `ActionURL.ts` builds LabKey-style URLs of the form `{contextPath}/{containerPath}/{controller}-{action}.view`. These two files are the only ones that actually touch the network.

**Layer 2 — Shared utilities.** `Utils.ts` provides `getCallbackWrapper` (the universal JSON response parser/error handler), callback resolution helpers (`getOnSuccess`/`getOnFailure`), and miscellaneous string/date/cookie helpers. `Filter.ts` and `filter/Types.ts` define the filter type system (~40 filter operators like `eq`, `gt`, `contains`, `in`, `between`, etc.) with URL encoding, validation, and multi-value support.

**Layer 3 — Domain modules.** Each module is a thin facade that calls `request()` with a `buildURL()` and wraps callbacks with `getCallbackWrapper()`. The modules and their responsibilities are enumerated below.

### Module Inventory

| Module               | Key Endpoints                                                                                                                                                                                                                                                                                                                                                       | Purpose                                           |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------- |
| **Query**            | selectRows, executeSql, insertRows, updateRows, deleteRows, saveRows, moveRows, truncateTable, getQueries, getSchemas, getQueryDetails, getQueryViews, saveQueryViews, selectDistinctRows, getDataViews, validateQuery, getServerDate                                                                                                                               | Core data CRUD and metadata                       |
| **Security**         | getContainers, createContainer, deleteContainer, moveContainer, getUsers, createNewUser, getRoles, getPolicy, savePolicy, getGroupPermissions, createGroup, deleteGroup, addGroupMembers, renameContainer, getModules, getFolderTypes, getReadableContainers, getUserPermissions, getSecurableResources, getSchemaPermissions, ensureLogin, getUsersWithPermissions | Container, user, group, and permission management |
| **Domain**           | create, get, getDomainDetails, save, drop, updateDomain, listDomains, getProperties, getPropertyUsages, validateNameExpressions, getDomainNamePreviews                                                                                                                                                                                                              | Schema/domain design management                   |
| **Experiment**       | lineage, saveBatch, saveBatches, loadBatch, loadBatches, saveRuns, loadRuns, resolve, createHiddenRunGroup, saveMaterials, setEntitySequence, getEntitySequence                                                                                                                                                                                                     | Experiment framework, lineage, runs, batches      |
| **Assay**            | getAssays, getNAbRuns, getStudyNabGraphURL, getStudyNabRuns                                                                                                                                                                                                                                                                                                         | Assay design retrieval, NAb assay data            |
| **Exp**              | Classes: ExpObject, Run, RunGroup, Data, Material, SampleSet, DataClass, Protocol                                                                                                                                                                                                                                                                                   | Object model for experiment entities              |
| **List**             | create                                                                                                                                                                                                                                                                                                                                                              | List creation (delegates to Domain.create)        |
| **Pipeline**         | getFileStatus, getPipelineContainer, getProtocols, startAnalysis                                                                                                                                                                                                                                                                                                    | Pipeline job management                           |
| **Report**           | createSession, deleteSession, execute, executeFunction, getSessions                                                                                                                                                                                                                                                                                                 | R/Python report execution                         |
| **Message**          | sendMessage                                                                                                                                                                                                                                                                                                                                                         | Email messaging                                   |
| **Storage**          | createStorageItem, updateStorageItem, deleteStorageItem                                                                                                                                                                                                                                                                                                             | Freezer Manager storage hierarchy                 |
| **Specimen**         | addSpecimensToRequest, addVialsToRequest, cancelRequest, getOpenRequests, getProvidingLocations, getRepositories, getRequest, getVialsByRowId, getVialTypeSummary, getSpecimenWebPartGroups, removeVialsFromRequest                                                                                                                                                 | Specimen/vial request management                  |
| **ParticipantGroup** | updateParticipantGroup                                                                                                                                                                                                                                                                                                                                              | Study participant group management                |
| **Visualization**    | (re-exports from query/Visualization)                                                                                                                                                                                                                                                                                                                               | Visualization metadata                            |
| **App**              | registerApp, loadApp, init                                                                                                                                                                                                                                                                                                                                          | Browser app registry (not relevant for Rust)      |

### Codebase Size

- ~227 exported functions
- ~282 exported types/interfaces/enums/classes
- ~7,500 lines of non-test TypeScript (before stripping browser-specific code)
- ~4,000–5,000 lines of meaningful API logic after removing browser artifacts

## What Can Be Dropped for a Rust Client

Several things in this codebase are browser-specific artifacts that have no place in a Rust HTTP client:

- **`App` module** — entirely about DOM app registration and hot module reload. Not an API.
- **DOM modules** (`dom/Assay.ts`, `dom/Query.ts`, `dom/Utils.ts`, `dom/Security.ts`) — browser UI helpers (file upload forms, alert dialogs, etc.).
- **`DOMWrapper` pattern** in Utils — stubs for browser-only UI functions (`alert`, `displayAjaxErrorResponse`, `onReady`, `onError`).
- **Cookie/session helpers** — `getCookie`, `setCookie`, `deleteCookie`, `getSessionID`.
- **`requiresScript`/`requiresCSS`** — dynamic script loading.
- **`MultiRequest`** — a callback-coordination utility. In Rust, `tokio::join!` or `futures::join_all` serve this purpose.
- **`downloadFile`** in Ajax — browser blob download via anchor tag click.
- **IE11 date parsing** — obviously.
- **Ext.js compatibility shims** — `DATEALTFORMATS`, `ensureBoxVisible`, `resizeToViewport`, etc.
- **The entire callback pattern** — the JS client uses `success`/`failure` callbacks everywhere. A Rust client would use `Result<T, E>` return types from async functions.
- **`getServerContext()` / `LABKEY` global** — the JS client reads configuration from a global `LABKEY` object injected by the server into the page. A Rust client would take configuration (base URL, credentials, container path) as constructor arguments.

## What Needs to Be Ported

The actual API surface that matters is the set of HTTP endpoints the client talks to and the request/response shapes it uses.

1. **URL construction** — `buildURL(controller, action, containerPath, params)` and `queryString(params)`. Straightforward string manipulation.

2. **HTTP transport** — Replace `XMLHttpRequest` with `reqwest`. Handle CSRF tokens, JSON content types, and authentication. The JS client uses a CSRF token from the page; a Rust client would likely use API keys or basic auth.

3. **Filter system** — The ~40 filter types with their URL suffixes, multi-value separators, and encoding rules. This is the most intricate piece to port but it's pure data transformation with no I/O. It maps cleanly to a Rust enum with methods.

4. **ContainerFilter enum** — 10 variants, trivial.

5. **~60–70 API endpoint functions** — Each one is a thin wrapper that builds a URL, constructs a JSON body or query params, makes an HTTP request, and deserializes the response. In Rust, each becomes an async method on a client struct that returns `Result<ResponseType, Error>`.

6. **Request/response types** — ~100+ TypeScript interfaces that describe the shapes of JSON request bodies and response payloads. These become Rust structs with `#[derive(Serialize, Deserialize)]`.

7. **Exp object model** — The `ExpObject`, `Run`, `RunGroup`, `Data`, `Material`, `SampleSet`, `DataClass` class hierarchy. In Rust these become structs (no inheritance needed — composition or trait-based dispatch works fine).

## Feasibility Assessment

This is a very achievable project for the following reasons.

**The pattern is extremely uniform.** Nearly every function in the codebase follows the same template: build a URL, optionally construct a JSON body, make an HTTP request, deserialize the JSON response. There are no WebSocket connections, no streaming protocols, no binary formats, no complex state machines. It's pure request-response JSON over HTTP.

**The type system is already well-defined.** The TypeScript interfaces give an almost 1:1 blueprint for Rust structs. Using `serde` for serialization, the types would be nearly mechanical translations.

**The filter system is the most complex piece, and it's still manageable.** It's a closed set of ~40 operators with well-defined URL encoding rules. A Rust enum with a `to_url_suffix()` method and associated validation logic handles this cleanly.

**Dependencies would be minimal.** The Rust client would need `reqwest` (HTTP), `serde`/`serde_json` (serialization), `url` (URL construction), `thiserror` (error types), and `tokio` (async runtime). That's it.

## Risks and Considerations

**Authentication.** The JS client piggybacks on browser session cookies and a CSRF token injected into the page. A Rust client needs to handle authentication differently. LabKey supports basic auth, API keys, and session-based auth. The auth story needs to be designed around the target server's configuration. This is the one area where mechanical translation from the JS is insufficient.

**Undocumented server behavior.** The JS client is tightly coupled to the server's behavior in ways that aren't always explicit. For example, the `wafEncode` function in `executeSql` does a BASE64 encoding to avoid WAF false positives. The `postParameters` fallback in `buildParameterMap` is server-specific. These edge cases will surface during testing against a real server.

**Response format versioning.** The `selectRows` and `executeSql` endpoints support multiple response formats via `requiredVersion` (8.3, 9.1, 13.2, 16.2, 17.1). The JS client has special handling for each. The Rust client should pick one version (probably 17.1, the latest) and stick with it, rather than supporting all of them.

**Scope.** Not everything needs to be ported on day one. The modules most likely to matter for a LIMS user are `Query` (selectRows, executeSql, insertRows, updateRows, deleteRows), `Security` (getContainers, getUsers), `Domain` (getDomainDetails), and `Experiment` (lineage). The `Specimen`, `NAb`, `Pipeline`, `Report`, and `Storage` modules can come later if needed.

## Proposed Architecture

```
labkey-rs/
├── Cargo.toml
├── src/
│   ├── lib.rs              // LabkeyClient struct, re-exports
│   ├── client.rs           // LabkeyClient: base_url, auth, reqwest::Client
│   ├── error.rs            // Error enum (thiserror)
│   ├── filter.rs           // FilterType enum, Filter struct
│   ├── query.rs            // select_rows, execute_sql, insert_rows, etc.
│   ├── security.rs         // get_containers, get_users, etc.
│   ├── domain.rs           // create, get_domain_details, save, etc.
│   ├── experiment.rs       // lineage, save_batch, load_runs, etc.
│   ├── assay.rs            // get_assays, etc.
│   ├── types/
│   │   ├── mod.rs
│   │   ├── container.rs    // Container, Project structs
│   │   ├── query.rs        // SelectRowsResponse, QueryColumn, etc.
│   │   ├── security.rs     // User, Group, Role, Policy, etc.
│   │   ├── domain.rs       // DomainDesign, etc.
│   │   ├── experiment.rs   // Run, RunGroup, LineageNode, etc.
│   │   └── filter.rs       // ContainerFilter enum
│   └── url.rs              // build_url, query_string
```

The `LabkeyClient` struct would hold a `reqwest::Client`, base URL, container path, and auth credentials. Each module's functions would be methods on the client (or on a sub-struct accessed via `client.query()`, `client.security()`, etc.).

A typical method would look something like:

```rust
pub async fn select_rows(
    &self,
    options: SelectRowsOptions,
) -> Result<SelectRowsResponse, LabkeyError> {
    let url = self.build_url(
        "query",
        "getQuery.api",
        options.container_path.as_deref(),
    );
    let params = options.to_query_params();
    let response = self.client.get(&url).query(&params).send().await?;
    let body: SelectRowsResponse = response.error_for_status()?.json().await?;
    Ok(body)
}
```

## Recommended Implementation Order

The following order builds out the infrastructure first, then adds endpoints in order of likely usage for a LIMS consumer.

1. **Client core** — `LabkeyClient` struct, URL construction, auth, error types
2. **Filter system** — `FilterType` enum, `Filter` struct, URL parameter encoding
3. **Query.selectRows** and **Query.executeSql** — the most commonly used read endpoints; forces the response type system to be built out
4. **Query.insertRows**, **Query.updateRows**, **Query.deleteRows** — core write operations
5. **Query.saveRows** — multi-command batch operations
6. **Security.getContainers**, **Security.getUsers** — basic container/user introspection
7. **Domain.getDomainDetails** — schema introspection
8. **Experiment.lineage** — lineage graph queries
9. **Remaining Query endpoints** — getQueries, getSchemas, getQueryDetails, selectDistinctRows, moveRows, truncateTable, etc.
10. **Remaining Security endpoints** — createContainer, permissions, groups, policies
11. **Assay, List, Domain (remaining)** — as needed
12. **Pipeline, Report, Message, Storage, Specimen, ParticipantGroup** — as needed

## Conclusion

This project is straightforward. The JS client is a thin, uniform HTTP wrapper over LabKey's REST API. The hardest part isn't the code — it's testing against a real LabKey server to verify that request/response shapes match expectations. Starting with `Query.selectRows` and `Query.executeSql` is recommended, since those are the most commonly used endpoints and will force the client infrastructure, filter system, and response types to be built out. From there, adding more endpoints is largely mechanical.
