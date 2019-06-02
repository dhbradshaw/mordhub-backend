use crate::app;
use actix_web::web::Json;

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
