//! wasi:http entrypoint: renders the dashboard in-process and serves it.

use adbir::{Config, HomeTemplate};
use askama::Template;
use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    sync::OnceLock,
};
use wstd::http::{Body, Error, Method, Request, Response, StatusCode};

const DEFAULT_CONFIG_PATH: &str = "/config.yaml";
const DEFAULT_PUBLIC_DIR: &str = "/public";

/// Cached for runtimes that reuse an instance across requests.
static INDEX: OnceLock<Result<String, String>> = OnceLock::new();

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn index() -> &'static Result<String, String> {
    INDEX.get_or_init(|| {
        let config_path = env_or("CONFIG_PATH", DEFAULT_CONFIG_PATH);
        eprintln!("Reading from {}", config_path);
        let config = Config::from_path(&config_path).map_err(|e| e.to_string())?;
        HomeTemplate::new(config)
            .render()
            .map_err(|e| e.to_string())
    })
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "txt" | "md" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

/// Resolves under `root`, rejecting any path that escapes it.
fn resolve(root: &str, request_path: &str) -> Option<PathBuf> {
    let decoded = percent_decode(request_path)?;
    let mut resolved = PathBuf::from(root);
    for component in Path::new(&decoded).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::RootDir => {}
            _ => return None,
        }
    }
    Some(resolved)
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes.get(i + 1..i + 3)?;
            let hex = std::str::from_utf8(hex).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn respond(status: StatusCode, content_type: &str, body: Vec<u8>) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", content_type)
        .body(body.into())?)
}

#[wstd::http_server]
async fn main(request: Request<Body>) -> Result<Response<Body>, Error> {
    if !matches!(request.method(), &Method::GET | &Method::HEAD) {
        return respond(
            StatusCode::METHOD_NOT_ALLOWED,
            "text/plain; charset=utf-8",
            b"method not allowed\n".to_vec(),
        );
    }
    let head = request.method() == Method::HEAD;
    let path = request.uri().path().to_string();
    eprintln!("{} {}", request.method(), path);

    let (status, content_type, body) = match path.as_str() {
        "/" | "/index.html" => match index() {
            Ok(html) => (
                StatusCode::OK,
                "text/html; charset=utf-8",
                html.clone().into_bytes(),
            ),
            Err(error) => {
                eprintln!("Failed to render dashboard: {}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "text/plain; charset=utf-8",
                    format!("failed to render dashboard: {error}\n").into_bytes(),
                )
            }
        },
        _ => {
            let public_dir = env_or("PUBLIC_DIR", DEFAULT_PUBLIC_DIR);
            match resolve(&public_dir, &path)
                .and_then(|file| fs::read(&file).ok().map(|b| (file, b)))
            {
                Some((file, bytes)) => (StatusCode::OK, content_type(&file), bytes),
                None => (
                    StatusCode::NOT_FOUND,
                    "text/plain; charset=utf-8",
                    b"not found\n".to_vec(),
                ),
            }
        }
    };

    respond(status, content_type, if head { Vec::new() } else { body })
}
