//! Embedded frontend static assets, built via npm and embedded at compile time.

include!(concat!(env!("OUT_DIR"), "/frontend.rs"));

/// Returns the embedded frontend static asset for the given path, or `None` if not found.
#[allow(dead_code)]
pub fn get_frontend_asset(path: &str) -> Option<&'static [u8]> {
    FRONTEND
        .iter()
        .find(|(k, _, _, _, _)| *k == path)
        .map(|(_, v, _, _, _)| *v)
}

/// Returns the embedded frontend static asset and its ETag for the given path.
pub fn get_frontend_asset_with_etag(path: &str) -> Option<(&'static [u8], &'static str)> {
    FRONTEND
        .iter()
        .find(|(k, _, _, _, _)| *k == path)
        .map(|(_, v, etag, _, _)| (*v, *etag))
}

/// Returns the best compressed variant (brotli > gzip) and its ETag, if smaller than uncompressed.
/// Returns `(data, encoding, etag)` — `encoding` is "" when no compressed variant exists.
pub fn get_frontend_asset_compressed(
    path: &str,
) -> Option<(&'static [u8], &'static str, &'static str)> {
    FRONTEND
        .iter()
        .find(|(k, _, _, _, _)| *k == path)
        .map(|(_, v, etag, brotli, gzip)| {
            if let Some((ref data, enc)) = *brotli {
                if !enc.is_empty() {
                    return (data.as_slice(), enc, *etag);
                }
            }
            if let Some((ref data, enc)) = *gzip {
                if !enc.is_empty() {
                    return (data.as_slice(), enc, *etag);
                }
            }
            (*v, "", *etag)
        })
}
