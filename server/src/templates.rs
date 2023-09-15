use askama::Template;
use axum::http::StatusCode;
use axum::response::{self, IntoResponse, Response};

pub struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => response::Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "hello.html")]
pub struct Hello<'a> {
    name: &'a str,
}

pub fn hello<'a>(name: &'a str) -> HtmlTemplate<Hello<'a>> {
    let hello_template = Hello { name };
    HtmlTemplate(hello_template)
}
