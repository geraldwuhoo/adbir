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
  - [Wasm container](#wasm-container)
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
| `logo` | URL to logo of the service | Optional |

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

There are two flavors of the Docker container:

1. Vanilla: `adbir:<tag>`. This version of the container just contains the webserver and nothing more.
1. Dashboard Icons: `adbir:<tag>-dashboard-icons`. This version of the container also bundles [dashboard-icons](https://github.com/walkxcode/dashboard-icons) served at `/icons` in the webserver.

Both flavors are published as a multi-platform index: `adbir:<tag>` resolves to
the `linux/amd64` image, or to the wasm image below on a wasm shim. The
individual platforms are also tagged `adbir:<tag>-amd64` and `adbir:<tag>-wasm`.

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

### Wasm container

`Dockerfile.wasm` builds a self-contained wasm OCI image: a single
[wasi:http](https://github.com/WebAssembly/wasi-http) component that renders the
dashboard in-process and serves it, with no shell and no bundled webserver.

The component lives in its own crate, `crates/adbir-serve`, built as a
`cdylib` rather than a `[[bin]]`. That is deliberate: a bin target also exports
`wasi:cli/run`, and a host that dispatches on the first entrypoint it finds --
runwasi's `containerd-shim-wasmtime` does -- then runs the component as a
command, with no `wasi:http` in its linker. A `cdylib` produces a reactor
component exporting `wasi:http/incoming-handler` alone, leaving nothing to
pick wrong.

```
$ buildah build --file Dockerfile.wasm --build-arg "ICONS=dashboard-icons" --tag adbir-wasm .
$
```

It needs a host implementing `wasi:http`, which in practice means a
wasmtime-based one -- WasmEdge does not implement the WASI 0.2 APIs. Both the
`wasm32-wasip2` target and `wasmtime` are provided by the flake's dev shell:

```
$ cargo build --release --target wasm32-wasip2 --package adbir-serve
$ wasmtime serve -S cli --dir ./::/ --env CONFIG_PATH=/config.yaml --env PUBLIC_DIR=/public \
    target/wasm32-wasip2/release/adbir_serve.wasm
Serving HTTP on http://0.0.0.0:8080/
$
```

`-S cli` is required: the component reads its config from the filesystem and
its settings from the environment, so the bare `wasi:http/proxy` world is not
enough. Shim hosts link the full `wasi:cli` surface alongside `wasi:http`
already.

Configuration is by environment variable only, since a component has no `argv`:

| Variable | Description | Default |
| --- | --- | --- |
| `CONFIG_PATH` | path to config file | `/config.yaml` |
| `PUBLIC_DIR` | directory of static assets to serve | `/public` |

Anything not matching `/` or `/index.html` is served from `PUBLIC_DIR`, which is
where the `-dashboard-icons` flavor puts `/icons`.

#### containerd

Put `containerd-shim-wasmtime-v1` on containerd's `PATH`, then register it as a
runtime. For containerd 2.x (config `version = 3`):

```toml
version = 3

[plugins.'io.containerd.cri.v1.runtime'.containerd.runtimes.wasmtime]
  runtime_type = 'io.containerd.wasmtime.v1'
```

On containerd 1.7 (`version = 2`) the plugin key is `io.containerd.grpc.v1.cri`.
The shim listens on `0.0.0.0:8080` and preopens the container rootfs, so
`CONFIG_PATH` and `PUBLIC_DIR` resolve without extra mounts.

#### Kubernetes

The shim implements no `Exec`, so `kubectl exec`, `exec` probes and
`lifecycle.postStart` hooks all fail against these pods -- anything the
dashboard needs on disk (a `logo.webp` for the root `image:` key, say) has to
be baked into the image or mounted. Use `httpGet` probes, not `exec` ones.

The image is `FROM scratch`, so the config has to come from a volume:

```yaml
---
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: wasmtime
handler: wasmtime
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: adbir
data:
  config.yaml: |
    title: My Dashboard
    services:
      - name: Media
        items:
          - name: Jellyfin
            url: https://jellyfin.example.com
            logo: /icons/svg/jellyfin.svg
            subtitle: Media server
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: adbir
spec:
  replicas: 1
  selector:
    matchLabels:
      app: adbir
  template:
    metadata:
      labels:
        app: adbir
    spec:
      runtimeClassName: wasmtime
      containers:
        - name: adbir
          image: registry.wuhoo.xyz/jerry/adbir:stable-wasm-dashboard-icons
          ports:
            - containerPort: 8080
          volumeMounts:
            - name: config
              mountPath: /config.yaml
              subPath: config.yaml
      volumes:
        - name: config
          configMap:
            name: adbir
---
apiVersion: v1
kind: Service
metadata:
  name: adbir
spec:
  selector:
    app: adbir
  ports:
    - port: 80
      targetPort: 8080
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
