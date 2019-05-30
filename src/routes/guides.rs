use crate::app::{self, PageTitle, State};
use crate::models::User;
use actix_web::{web, HttpResponse};

pub fn list(user: Option<User>, state: web::Data<State>) -> Result<HttpResponse, app::Error> {
    let ctx = State::tera_context(PageTitle::GuideList, user);
    state.render("guides/list.html", ctx)
}

pub fn single(
    path: web::Path<String>,
    user: Option<User>,
    state: web::Data<State>,
) -> Result<HttpResponse, app::Error> {
    let (template, title) = match path.as_str() {
        "test" => ("guides/gen/test.html", "Test"),
        _ => return Err(app::Error::NotFound),
    };

    let ctx = State::tera_context(PageTitle::GuideSingle(title), user);

    state.render(template, ctx)
}
