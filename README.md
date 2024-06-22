<div align="center">
  <h1>adbir</h1>
  <b>A</b>nother <b>D</b>ashboard <b>B</b>ut <b>I</b>n <b>R</b>ust
</div>

- [About](#about)
- [Configuring](#configuring)
  - [Root](#root)
  - [`.services`](#services)
  - [`.services.items`](#servicesitems)
  - [Example](#example)
- [Running](#running)
  - [Pre-built binary](#pre-built-binary)
  - [Docker container](#docker-container)
  - [Building from source](#building-from-source)

## About

A dashboard inspired by [Homer](https://github.com/bastienwirtz/homer), both in
design and configuration.

While I enjoy the configurability and relatively minimalist design of Homer, I
dislike its hard dependency on JavaScript to render the page. This is a stripped
down rewrite of Homer without any JavaScript.

## Configuring

Runtime arguments to configure the program can be provided either with CLI
arguments or environment variables.

```
Usage: adbir [OPTIONS]

Options:
      --out-dir <OUT_DIR>          directory to output generated resources [env: OUT_DIR=] [default: ./out]
      --config-path <CONFIG_PATH>  path to config file [env: CONFIG_PATH=] [default: ./config.yaml]
  -h, --help                       Print help
  -V, --version                    Print version
```

The config file is inspired by a subset of Homer configuration options.

### Root

| Field | Description | Required |
| --- | --- | --- |
| `title` | Title of the dashboard | Required |
| `subtitle` | Subtitle of the dashboard displayed underneath title | Optional |
| `image` | URL to image to display at the top of the page (e.g. logo) | Optional |
| `services` | List of service groups | Required |

### `.services`

| Field | Description | Required |
| --- | --- | --- |
| `name` | Name of service group | Required |
| `items` | List of service groups | Required |

### `.services.items`

| Field | Description | Required |
| --- | --- | --- |
| `name` | Name of service | Required |
| `subtitle` | Description of service | Required |
| `url` | URL to the service | Required |
| `logo` | Logo of the service to display | Required |

### Example

```yaml
title: Dashboard
subtitle: A list of services
image: logo.webp

services:
  - name: Application group
    items:
      - name: Selfhosted app
        url: https://selfhosted.local
        logo: /icons/svg/files.svg
        subtitle: Cool selfhosted app
      - name: Another selfhosted app
        url: https://otherselfhosted.local
        logo: /icons/svg/another.svg
        subtitle: Another cool selfhosted app
  - name: Another application group
    items:
      - name: Other app
        url: https://otherapp.local
        logo: /icons/svg/other.svg
        subtitle: Another app
```

## Running

### Pre-built binary

Download the latest binary from the releases. Then, run:

```
$ adbir
Started with args: Args { out_dir: "./out", config_path: "./config.yaml" }
Reading from ./config.yaml
Opening output directory file
Rending and writing template to output file
$
```

This will generate a static `index.html` in the desired output directory. Upload
this file to your desired webserver to serve.

### Docker container

The Docker container generates the static files on startup and serves them
using a bundled lightweight webserver (`darkhttpd`). The webserver serves files
from `/public` in the container.

An example `docker-compose.yaml`.

```yaml
---
version: '2'

services:
  adbir:
    image: registry.wuhoo.xyz/jerry/adbir:v0.1.2
    environment:
      OUT_DIR: /public
      CONFIG_PATH: /config.yaml
    ports:
      - "8080:8080"
    volumes:
      - ./config.yaml:/config.yaml
```

Then run:

```
$ docker-compose up -d
$
```

### Building from source

```
$ cargo run --release
    Finished release [optimized] target(s) in 0.02s
     Running `target/release/adbir`
Started with args: Args { out_dir: "./out", config_path: "./config.yaml" }
Reading from ./config.yaml
Opening output directory file
Rending and writing template to output file
$
```
