use askama::Template; // bring trait in scope
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
pub struct HtmlTemplate<T>(pub T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

#[derive(Template)] // this will generate the code...
#[template(path = "index.html")] // using the template in this path, relative to the `templates` dir in the crate root
pub struct IndexTemplate {
    pub name: String, // the field name should match the variable name
                      // in your template
}

#[derive(Template)] // this will generate the code...
#[template(path = "dashboard_overview.html")] // using the template in this path, relative to the `templates` dir in the crate root
pub struct DashboardOverviewTemplate;

#[derive(Template)] // this will generate the code...
#[template(path = "404.html")] // using the template in this path, relative to the `templates` dir in the crate root
pub struct Error404Template;
