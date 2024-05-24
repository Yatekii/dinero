use axum::response::Html;

pub async fn handler() -> Html<String> {
    Html(std::fs::read_to_string("frontend/index.html").unwrap())
}
