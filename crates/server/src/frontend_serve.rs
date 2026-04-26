use actix_web::{http::header, HttpRequest, HttpResponse};
use mime_guess2::MimeGuess;

use crate::frontend;

fn cache_control_header(path: &str) -> &'static str {
    if path == "index.html" {
        "no-cache, no-store, must-revalidate"
    } else if path.ends_with(".js")
        || path.ends_with(".css")
        || path.ends_with(".woff2")
        || path.ends_with(".woff")
        || path.ends_with(".ttf")
        || path.ends_with(".otf")
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
        || path.ends_with(".webp")
        || path.ends_with(".avif")
        || path.ends_with(".map")
    {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    }
}

fn accepts_encoding(req: &HttpRequest, target: &str) -> bool {
    let header_val = match req.headers().get(header::ACCEPT_ENCODING) {
        Some(v) => v.to_str().unwrap_or(""),
        None => return false,
    };
    for part in header_val.split(',') {
        let part = part.trim();
        let enc = part.split(';').next().unwrap_or(part).trim();
        if enc.eq_ignore_ascii_case(target) || enc == "*" {
            return true;
        }
    }
    false
}

fn content_type_for_path(path: &str) -> String {
    if let Some(ext) = path.rsplit('.').next() {
        match ext {
            "html" | "htm" => return "text/html; charset=utf-8".into(),
            "js" | "mjs" => return "application/javascript; charset=utf-8".into(),
            "jsx" | "ts" | "tsx" => return "text/plain; charset=utf-8".into(),
            "css" => return "text/css; charset=utf-8".into(),
            "json" => return "application/json; charset=utf-8".into(),
            "svg" => return "image/svg+xml".into(),
            "png" => return "image/png".into(),
            "jpg" | "jpeg" => return "image/jpeg".into(),
            "gif" => return "image/gif".into(),
            "webp" => return "image/webp".into(),
            "ico" => return "image/x-icon".into(),
            "woff" => return "font/woff".into(),
            "woff2" => return "font/woff2".into(),
            "ttf" => return "font/ttf".into(),
            "otf" => return "font/otf".into(),
            "txt" => return "text/plain; charset=utf-8".into(),
            "xml" => return "application/xml; charset=utf-8".into(),
            _ => {}
        }
    }
    MimeGuess::from_path(path).first_or_octet_stream().to_string()
}

fn build_asset_response(
    req: &HttpRequest,
    data: &[u8],
    etag: &str,
    path: &str,
    cc: &str,
    content_encoding: &str,
) -> HttpResponse {
    if let Some(if_none_match) = req.headers().get(header::IF_NONE_MATCH) {
        if let Ok(client_etag) = if_none_match.to_str() {
            if client_etag == etag {
                let mut resp = HttpResponse::NotModified();
                resp.insert_header(("Cache-Control", cc));
                resp.insert_header(("ETag", etag));
                if !content_encoding.is_empty() {
                    resp.insert_header(("Content-Encoding", content_encoding));
                }
                return resp.finish();
            }
        }
    }

    let mime = content_type_for_path(path);
    let mut resp = HttpResponse::Ok();
    resp.content_type(mime);
    resp.insert_header(("Cache-Control", cc));
    resp.insert_header(("ETag", etag));
    if !content_encoding.is_empty() {
        resp.insert_header(("Content-Encoding", content_encoding));
    }
    resp.body(data.to_vec())
}

pub async fn serve_frontend(req: HttpRequest, path: actix_web::web::Path<String>) -> HttpResponse {
    let path = path.into_inner();
    let path_str = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path.as_str()
    };
    let cc = cache_control_header(path_str);

    match frontend::get_frontend_asset_compressed(path_str) {
        Some(r) => {
            let (data, encoding, etag) = r;
            if !encoding.is_empty() && accepts_encoding(&req, encoding) {
                build_asset_response(&req, data, etag, path_str, cc, encoding)
            } else {
                build_asset_response(&req, data, etag, path_str, cc, "")
            }
        }
        None => {
            match frontend::get_frontend_asset_with_etag("index.html") {
                Some((data, etag)) => build_asset_response(&req, data, etag, "index.html", cc, ""),
                None => HttpResponse::NotFound().finish(),
            }
        }
    }
}
