use crate::{
    app::{self, ActiveLink, State, TmplBase},
    models::User,
};
use actix_web::HttpResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "guides/list.html")]
struct GuidesList {
    base: TmplBase,
}

pub fn list(user: Option<User>) -> Result<HttpResponse, app::Error> {
    State::render(GuidesList {
        base: TmplBase::new(user, ActiveLink::Guides),
    })
}

// pub fn single(
//     path: web::Path<String>,
//     user: Option<User>,
//     state: web::Data<State>,
// ) -> Result<HttpResponse, app::Error> {
//     let (template, title) = match path.as_str() {
//         "test" => ("guides/gen/test.html", "Test"),
//         _ => return Err(app::Error::NotFound),
//     };

//     let ctx = State::tera_context(PageTitle::GuideSingle(title), user);

//     state.render(template, ctx)
// }
