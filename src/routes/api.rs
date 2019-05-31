use crate::app::{self, PageTitle, State};
use crate::models::User;
use actix_web::{web::Json, HttpResponse};

#[derive(Serialize)]
pub struct TestResponse {
    foo: String,
    bar: i32,
    baz: bool,
}

pub fn test() -> Result<Json<TestResponse>, app::Error> {
    Ok(Json(TestResponse {
        foo: "hello".to_owned(),
        bar: 123,
        baz: false,
    }))
}
