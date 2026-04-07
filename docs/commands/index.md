# Commands Overview

```
keentools-cloud [OPTIONS] <COMMAND>
```

## Global Options

| Flag                     | Env Var                | Description                              |
| ------------------------ | ---------------------- | ---------------------------------------- |
| `--api-url <URL>`        | `KEENTOOLS_API_URL`    | API base URL (**required**)              |
| `--token <TOKEN>`        | `KEENTOOLS_API_TOKEN`  | API authentication token                 |
| `--output <FORMAT>`      | —                      | Output format: `human` \| `json`         |
| `-h, --help`             | —                      | Print help                               |
| `-V, --version`          | —                      | Print version                            |

## Command List

| Command       | Description                                      |
| ------------- | ------------------------------------------------ |
| `auth`        | Manage authentication (login/logout/status)      |
| `init`        | Initialize a reconstruction job                  |
| `upload`      | Upload photos to presigned S3 URLs               |
| `process`     | Start 3D reconstruction                          |
| `status`      | Check reconstruction status                      |
| `download`    | Download the completed 3D model                  |
| `info`        | Get camera metadata for an avatar                |
| `run`         | Full pipeline: init → upload → process → download |
| `ephemeral`   | Zero-retention reconstruction pipeline           |
| `schema`      | Dump CLI/API schema (for agent introspection)    |
| `self-update` | Update to the latest release                     |

Commands that do **not** require `--api-url` or `--token`: `auth`, `schema`, `self-update`.
