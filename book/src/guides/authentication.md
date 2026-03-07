# Authentication

Every `LabkeyClient` requires a `Credential` that determines how requests are authenticated. There are three credential types — API key, basic auth, and guest — plus a convenience method for reading credentials from a `.netrc` file. This page covers all of them and discusses when to use each.

## API key

API keys are the recommended credential type for most use cases. They're scoped to a single user, can be revoked independently of the user's password, and don't require sending the user's actual password over the wire.

To create an API key, log into your LabKey server, navigate to your user profile, and look for the API Keys section. LabKey's [API Keys documentation](https://www.labkey.org/Documentation/wiki-page.view?name=apikey) has the full details.

Once you have a key, pass it as an environment variable:

```rust,no_run
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

let api_key = std::env::var("LABKEY_API_KEY")
    .expect("LABKEY_API_KEY must be set");

let config = ClientConfig::new(
    "https://labkey.example.com/labkey",
    Credential::ApiKey(api_key),
    "/MyProject",
);
let client = LabkeyClient::new(config)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Under the hood, the client sends API keys as HTTP Basic authentication with the username `"apikey"` and the key as the password, which is the convention LabKey expects.

## Basic auth (email and password)

If your server doesn't support API keys (older versions) or you need to authenticate as a specific user without generating a key, you can use email and password directly:

```rust,no_run
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

let email = std::env::var("LABKEY_EMAIL")
    .expect("LABKEY_EMAIL must be set");
let password = std::env::var("LABKEY_PASSWORD")
    .expect("LABKEY_PASSWORD must be set");

let config = ClientConfig::new(
    "https://labkey.example.com/labkey",
    Credential::Basic { email, password },
    "/MyProject",
);
let client = LabkeyClient::new(config)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Basic auth sends the email and password with every request. Prefer API keys when possible — they can be rotated without changing the user's password, and revoking a key doesn't lock the user out of the web interface.

## Guest access

For servers that allow anonymous access, the `Guest` credential sends requests without an `Authorization` header:

```rust,no_run
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

let config = ClientConfig::new(
    "https://labkey.example.com/labkey",
    Credential::Guest,
    "/Public",
);
let client = LabkeyClient::new(config)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The server grants only guest-level permissions, so this is only useful for reading publicly shared data. Most production servers don't enable guest access at all.

## Reading credentials from `.netrc`

If you work with multiple LabKey servers, maintaining environment variables for each one gets tedious. The `.netrc` file format (also used by curl, git, and other tools) lets you store credentials per host in a single file.

A `.netrc` file lives at `~/.netrc` on Unix or `~/_netrc` on Windows and looks like this:

```text
machine labkey.example.com
  login alice@example.com
  password s3cret

machine labkey-staging.example.com
  login alice@example.com
  password staging-pw
```

The `Credential::from_netrc` method reads this file and returns a `Credential::Basic` for the matching host:

```rust,no_run
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

let credential = Credential::from_netrc("labkey.example.com")
    .expect("no .netrc entry for labkey.example.com");

let config = ClientConfig::new(
    "https://labkey.example.com/labkey",
    credential,
    "/MyProject",
);
let client = LabkeyClient::new(config)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The host argument should be just the hostname (no scheme, no path). The method looks for `~/.netrc` first, then `~/_netrc`, and returns an error if neither exists or if no entry matches the given host.

> **File permissions matter.** On Unix, your `.netrc` file should be readable only by you (`chmod 600 ~/.netrc`). Some tools refuse to read it otherwise, and leaving credentials world-readable is a security risk.

## Choosing a credential type

For **automated pipelines and CI/CD**, use API keys via environment variables. They're easy to rotate, easy to revoke, and don't expose the user's password.

For **local development** across multiple servers, `.netrc` is convenient. You configure it once and every tool that understands the format (curl, this client, etc.) picks it up automatically.

For **one-off scripts** where you're the only user, basic auth with environment variables works fine. Just don't hardcode the password in source code.

For **reading public data** on servers that allow it, guest access avoids credential management entirely.

## Additional client configuration

The `ClientConfig` struct has a few optional settings beyond credentials that are worth knowing about:

```rust,no_run
use labkey_rs::{ClientConfig, Credential};

let api_key = std::env::var("LABKEY_API_KEY")?;

let config = ClientConfig::new(
    "https://labkey.example.com/labkey",
    Credential::ApiKey(api_key),
    "/MyProject",
)
.with_user_agent("my-pipeline/1.0")
.with_proxy_url("http://proxy.example.com:8080")
.with_accept_self_signed_certs(true)
.with_csrf_token("my-csrf-token");
# Ok::<(), Box<dyn std::error::Error>>(())
```

`with_user_agent` sets a custom `User-Agent` header (the default is `labkey-rs/{version}`). `with_proxy_url` routes all requests through an HTTP proxy. `with_accept_self_signed_certs` disables TLS certificate validation, which is useful for development servers with self-signed certificates but should never be used in production. `with_csrf_token` attaches a CSRF token to every request — this is rarely needed with API key authentication, since LabKey typically skips CSRF validation for API keys.
