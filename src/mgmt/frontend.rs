#[cfg(not(debug_assertions))]
mod embedded {
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "web/dist/"]
    pub struct Assets;
}

#[cfg(not(debug_assertions))]
pub async fn static_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    use axum::response::IntoResponse;

    let path = uri.path().trim_start_matches('/');

    if let Some(file) = embedded::Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        (
            [(axum::http::header::CONTENT_TYPE, mime.as_ref().to_owned())],
            file.data,
        )
            .into_response()
    } else if let Some(index) = embedded::Assets::get("index.html") {
        let mime = mime_guess::from_path("index.html").first_or_octet_stream();
        (
            [(axum::http::header::CONTENT_TYPE, mime.as_ref().to_owned())],
            index.data,
        )
            .into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}
