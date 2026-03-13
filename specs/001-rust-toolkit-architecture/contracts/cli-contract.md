# CLI Contract: open-tethyr

**Binary name**: `open-tethyr`
**Crate name**: `open-tethyr-cli`

## Global Options

```
open-tethyr [OPTIONS] <COMMAND>

Options:
  -v, --verbose    Increase log verbosity (repeatable: -v info, -vv debug, -vvv trace)
  -q, --quiet      Suppress non-error output
  -h, --help       Print help
  -V, --version    Print version
```

## Commands

### generate

Generate AX 1.0 records from YAML configuration.

```
open-tethyr generate [OPTIONS] --config <PATH> --output <PATH>

Options:
  -c, --config <PATH>    Path to YAML agent configuration file (required)
  -o, --output <PATH>    Output directory for well-known file structure (required)
      --validate         Validate generated records against AX 1.0 spec
  -h, --help             Print help
```

**Input**: YAML configuration file (AgentConfig format)
**Output**: `<output>/.well-known/agent-exchange.json`
**Exit codes**:
- 0: Success
- 1: Configuration file not found or unreadable
- 2: YAML parsing error
- 3: Validation error (when --validate is set)
- 4: Output write error

**stdout**: Path to generated file (normal mode) or JSON summary (with -v)
**stderr**: Error messages, warnings for deprecated fields

### validate

Validate an existing AX record file against AX 1.0 specification.

```
open-tethyr validate <PATH>

Arguments:
  <PATH>    Path to AX JSON file to validate

Options:
  -h, --help    Print help
```

**Input**: AX JSON file path
**Output**: Validation result to stdout
**Exit codes**:
- 0: Valid
- 1: File not found or unreadable
- 2: JSON parsing error
- 3: Validation failures (details printed to stderr)

**stdout**: "Valid AX 1.0 record" (success) or validation report
**stderr**: Each validation error on a separate line with field path and description

### discover

Test agent discovery for a target domain.

```
open-tethyr discover [OPTIONS] <DOMAIN>

Arguments:
  <DOMAIN>    Target domain to discover agents from

Options:
      --cache <URL>      Use specific cache server URL (skips DNS discovery)
      --direct           Skip cache, fetch directly from domain
      --json             Output results as JSON
      --timeout <SECS>   Request timeout in seconds (default: 30)
  -h, --help             Print help
```

**Input**: Domain name
**Output**: Discovered agents to stdout
**Exit codes**:
- 0: Agents discovered successfully
- 1: No agents found
- 2: Network error (DNS, HTTP)
- 3: Invalid AX record at endpoint

**stdout**: Human-readable agent list (default) or JSON (with --json)
**stderr**: Discovery progress, warnings, errors

### serve

Start the cache server.

```
open-tethyr serve [OPTIONS]

Options:
  -d, --domain <DOMAIN>    Server domain (required unless --config provided)
  -p, --port <PORT>        Listen port (default: 8080)
  -c, --config <PATH>      Path to server configuration YAML file
      --max-entries <N>     Maximum cache entries (default: 10000)
      --ttl <SECS>          Default TTL in seconds (default: 3600)
  -h, --help                Print help
```

**Input**: Configuration via flags or YAML file
**Output**: Server logs to stderr, HTTP API on configured port
**Exit codes**:
- 0: Clean shutdown
- 1: Configuration error
- 2: Port binding error
- 3: Runtime error

**stderr**: Structured logs (JSON format with tracing fields)

## Environment Variables

| Variable | Description | Overrides |
|----------|-------------|-----------|
| OPEN_TETHYR_LOG | Log level filter (e.g., "info", "debug,open_tethyr=trace") | --verbose flag |
| OPEN_TETHYR_CONFIG | Default config file path | --config flag |
| OPEN_TETHYR_DOMAIN | Server domain | --domain flag |
| OPEN_TETHYR_PORT | Server port | --port flag |

**Precedence**: CLI flags > Environment variables > Config file > Built-in defaults
