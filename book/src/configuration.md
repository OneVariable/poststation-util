# Configuration

On first run, `poststation` will create a configuration file in the default
data storage directory. This path will be printed on first run.

Additionally, you can use the `poststation-cli folder` to show the location
of the data storage directory, including the path to the configuration file.

```sh
$ poststation-cli folder
Poststation Folder Information:
===============================
Folder:         "/Users/username/Library/Application Support/com.onevariable.onevariable.poststation"
CA Certificate: "/Users/username/Library/Application Support/com.onevariable.onevariable.poststation/ca-cert.pem"
Configuration:  "/Users/username/Library/Application Support/com.onevariable.onevariable.poststation/poststation-config.toml"
```

For certain security settings, you may need to use the "CA Certificate" listed
here in order to establish secure connections. It is NOT recommended to add this
certificate to your operating system's certificate store or the one used by
your web browser.

## Default Contents

The default configuration file currently contains the following:

```toml
##
## Poststation Configuration File
##

# # `apis.sdk`
#
# This section is used for the binary SDK. This section is required.
[apis.sdk]
## API SDK Security options - pick ONE:

# Insecure, no encryption, only local connections will be allowed
# security.insecure        = {}

# Self-signed CA certificate. Global connections will be allowed, clients
#   on other machines will need a copy of the generated CA Certificate
#
# This is the default and recommended option.
# security.tls-self-signed = {}

## API SDK Listener options - pick ONE

# Listen only to connections on `localhost`
#
# This is the default option.
# listener.local-only = { port = 51837 }

# Listen to the given ipv4/ipv6 socket address. Use `0.0.0.0` to listen
#   on all interfaces. This option is not allowed when "Insecure" security
#   is selected.
# listener.global     = { socket_addr = "0.0.0.0:51837" }

# # `apis.http`
#
# This section is used for the REST API. This section is optional,
# and when omitted the REST API is disabled.
#
# When `insecure` security is selected, only `local-only` mode
# is allowed.
# [apis.http]
# ## REST API Security options - pick ONE:

# Insecure, no encryption, only local connections will be allowed
# security.insecure        = {}

# Self-signed CA certificate. Global connections will be allowed, clients
#   on other machines will need a copy of the generated CA Certificate
#
# This is the default and recommended option.
# security.tls-self-signed = {}

# ## Listener options
# listener.local-only = { port = 4444 } # default
# listener.global     = { socket_addr = "0.0.0.0:1235" }

# # `storage`
#
# This section is used to control local storage options. This section
# is optional.
# [storage]

# There are no configuration options for this yet.

# # `experimental`
#
# This section is used to control experimental, unstable features. This
# section is subject to change without stability guarantees
# [experimental]

# There are no configuration options for this yet.
```

## The `apis` section

The `apis` section contains public interfaces presented by the poststation
server. These are used to interact with the attached devices from your host
PC(s).

### The `apis.sdk` subsection

This section controls the SDK API. This API is used by the [`poststation-sdk`]
crate, as well as the [`poststation-cli`] tool.

[`poststation-sdk`]: https://docs.rs/poststation-sdk/latest/poststation_sdk/
[`poststation-cli`]: https://github.com/OneVariable/poststation-util/tree/main/tools/poststation-cli

By default, this API will be configured to:

* Serve ONLY to the local machine, on port 51837
* Serve using TLS encryption

Note that poststation will refuse to serve outside of the local machine, unless
TLS encryption is enabled.

If you wanted to serve on any interface, using TLS encryption, you could
use the following configuration:

```toml
[apis.sdk]
security.tls-self-signed = {}
listener.global          = { socket_addr = "0.0.0.0:51837" }
```

If you wanted to serve only locally, with no encryption, you could use
the following configuration:

```toml
[apis.sdk]
security.insecure   = {}
listener.local-only = { port = 51837 }
```

### The `apis.http` section

This section controls the HTTP/REST API. This API can be used by other languages
such as Python to communicate with the poststation server.

For example requests using `curl`, please see the [`poststation-api-icd` docs].

[`poststation-api-icd` docs]: https://docs.rs/poststation-api-icd/latest/poststation_api_icd/rest/index.html

By default, this API will be configured to:

* Serve ONLY to the local machine, on port 4444
* Serve using TLS encryption

Note that poststation will refuse to serve outside of the local machine, unless
TLS encryption is enabled.

If you wanted to serve on any interface, using TLS encryption, you could
use the following configuration:

```toml
[apis.http]
security.tls-self-signed = {}
listener.global          = { socket_addr = "0.0.0.0:4444" }
```

If you wanted to serve only locally, with no encryption, you could use
the following configuration:

```toml
[apis.http]
security.insecure   = {}
listener.local-only = { port = 4444 }
```
