use pulldown_cmark::{Options, Parser};
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::Path,
};

struct Info {
    name: &'static str,
    dst_dir: &'static str,
    typename: &'static str,
}

impl Info {
    pub fn new(a: &'static str, b: &'static str, c: &'static str) -> Self {
        Self {
            name: a,
            dst_dir: b,
            typename: c,
        }
    }
}

// Compile markdown files in the markdown folder into templates for the
// templates folder

fn main() -> io::Result<()> {
    if std::env::var("SKIP_BUILDRS").unwrap_or("0".to_string()) == "1" {
        return Ok(());
    }

    let mut askama_structs =
        String::from("use askama::Template;\nuse actix_web::web;\nuse crate::app::{TmplBase, ActiveLink, State};");
    let mut askama_scope = String::from(
        "\npub fn scope() -> actix_web::Scope {
    web::scope(\"/guides/\")",
    );

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let mut mappings = HashMap::new();
    mappings.insert(
        "markdown/guides/",
        Info::new("test", "templates/guides/gen/", "GuideTest"),
    );

    for (src_dir, info) in mappings {
        let src_path = Path::new(src_dir).join(info.name).with_extension("md");
        let dst_path = Path::new(info.dst_dir)
            .join(info.name)
            .with_extension("html");

        let _ = fs::create_dir(&info.dst_dir); // Create output folder if it doesn't already exist

        println!("cargo:rerun-if-changed={}", src_path.to_str().unwrap()); // Tell cargo to watch this file

        let markdown = fs::read_to_string(&src_path)?;
        let parser = Parser::new_ext(&markdown, options);

        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);

        let mut out = fs::File::create(dst_path)?;
        out.write_all("{% extends \"index.html\" %}\n\n{% block content %}\n".as_bytes())?;
        out.write_all(html.as_bytes())?;
        out.write_all("{% endblock %}\n".as_bytes())?;

        askama_structs += &format!(
            "\n
#[derive(Template)]
#[template(path = \"guides/gen/{}.html\")]
struct {} {{
    base: TmplBase,
}}",
            info.name, info.typename
        );

        askama_scope += &format!(
            "\
        \n        \
        .service(
            web::resource(\"/{}\")
                .route(
                    web::get()
                    .to(|user: Option<crate::models::User>|
                        State::render({} {{
                            base: TmplBase::new(user, ActiveLink::Guides)
                        }})
                    )
                )
        )",
            info.name, info.typename
        );
    }

    askama_scope += "\n}";

    let mut out_rust = fs::File::create("src/routes/gen/guides.rs")?;

    out_rust.write_all(format!("{}\n{}\n", askama_structs, askama_scope).as_bytes())?;

    Ok(())
}
