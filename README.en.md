# execenv

`execenv` is a small Unix-oriented entrypoint wrapper for containers.

It renders `${VAR}` placeholders in one or more files using values from the current environment, then can replace itself with another process via `exec(2)`.

This is useful when an application expects a static configuration file, but the final values are only known at container startup.

## Motivation

`execenv` is designed primarily for container images where configuration must be injected at runtime rather than build time.

A common case is a frontend or static distroless application image that is reused across multiple environments, such as development, staging, and production. Instead of rebuilding the image for each environment, you can keep a template in the image and render the final configuration when the container starts.

## Features

- Render `${VAR}` placeholders from environment variables.
- Process one or more files in a single run.
- Rewrite files in place or render an input file into a separate output file.
- Choose how missing environment variables are handled.
- Write through a temporary file and replace the target only after rendering succeeds.
- Optionally `exec` into the final application process.
- Intended for Linux/Unix containers.

## Limitations

> [!IMPORTANT]
> - The default read buffer size is 256 KiB.
> - The maximum placeholder variable name length is 4 KiB. Longer unfinished placeholders are preserved as plain text.
> - The output file inherits permissions from the input file.
> - Only Unix-like systems are supported.

## Installation

Build from source:

```sh
cargo build --release
```

The optimized binary will be available at:

```sh
target/release/execenv
```

The project requires **Rust 1.91** or newer.

## Usage

```sh
execenv --files <file> [<file> ...] [--missing-env <mode>] [--exec <command> [args...]]
```

Short flags are also available:

```sh
execenv -f <file> [-m <mode>] [-e <command> [args...]]
```

### File arguments

A bare file path is rendered in place:

```sh
execenv --files /app/config.yaml
```

A mapping renders one file into another:

```sh
execenv --files /templates/config.yaml=/app/config.yaml
```

Multiple files can be rendered in one run:

```sh
execenv --files /app/config.yaml /app/worker.yaml
```

or with mappings:

```sh
execenv --files \
  /templates/app.yaml=/app/config.yaml \
  /templates/worker.yaml=/app/worker.yaml
```

## Missing variables

`--missing-env` controls what happens when a file references an environment variable that is not set.

Available modes:

| Mode | Behavior |
| --- | --- |
| `empty` | Replace the placeholder with an empty string. This is the default. |
| `keep` | Leave the original `${VAR}` placeholder unchanged. |
| `error` | Fail immediately and do not start the command. |

Example:

```sh
execenv --files /app/config.yaml --missing-env error
```

For production environments, `--missing-env error` is usually the safest option because it prevents the application from starting with an incomplete configuration.

## Executing a command

When `--exec` is provided, `execenv` renders all requested files first. If rendering succeeds, it replaces itself with the given command using `exec(2)`.

```sh
execenv \
  --files /templates/nginx.conf=/etc/nginx/nginx.conf \
  --missing-env error \
  --exec nginx -g "daemon off;"
```

Everything after `--exec` is treated as the command and its arguments, including values that start with `-`.

If rendering fails, the command is not started.

## Example

Template file:

```yaml
server:
  host: ${APP_HOST}
  port: ${APP_PORT}
```

Run:

```sh
APP_HOST=0.0.0.0 APP_PORT=8080 \
execenv --files /templates/config.yaml=/app/config.yaml
```

Rendered output:

```yaml
server:
  host: 0.0.0.0
  port: 8080
```

## Docker example

```dockerfile
FROM chainguard/nginx:latest

COPY target/release/execenv /usr/bin/execenv
COPY templates/nginx.conf /templates/nginx.conf

ENTRYPOINT ["/usr/bin/execenv"]
CMD ["--files", "/templates/nginx.conf=/etc/nginx/nginx.conf", "--missing-env", "error", "--exec", "/usr/sbin/nginx", "-c", "/etc/nginx/nginx.conf", "-e", "/dev/stderr", "-g", "daemon off;"]
```

## Placeholder syntax

Only the `${VAR}` form is substituted.

Examples:

```text
${DATABASE_URL}
${APP_PORT}
${REDIS_HOST}
```

Other shell-style forms are not expanded:

```text
$VAR
${VAR:-default}
${VAR-default}
```

## Security notes

`execenv` does not evaluate shell expressions and does not execute environment variable values as code. The `--exec` command is executed directly as a program with arguments, not through `sh -c`.

However, rendering secrets from environment variables into files can make those secrets easier to expose. Be careful when writing generated configuration files to shared volumes, persistent storage, logs, support bundles, or locations readable by other processes.

Recommended practices:

- Treat templates and file paths as trusted input.
- Run containers as a non-root user when possible.
- Give write access only to the directory where rendered files should be created.
- Prefer `--missing-env error` in production.
- Avoid writing secrets to persistent or shared volumes unless necessary.

## Platform support

`execenv` is intended for Unix-like systems because `--exec` relies on `exec(2)`. The primary target is Linux containers.

## Development

Run tests:

```sh
cargo test
```

Check formatting:

```sh
cargo fmt --check
```

Run Clippy:

```sh
cargo clippy --all-targets --all-features -- -D warnings
```

Build an optimized binary:

```sh
cargo build --release
```

## License

Licensed under either of:

- MIT
- Apache-2.0
