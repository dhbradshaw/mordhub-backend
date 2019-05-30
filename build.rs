use pulldown_cmark::{Options, Parser};
use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};

fn main() -> io::Result<()> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let guide_in_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("markdown/guides");
    let guide_out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/guides/gen");

    for guide in fs::read_dir("markdown/guides")? {
        let guide = guide?;

        let path = guide.path();

        let mut src = fs::File::open(&path)?;
        let mut src_data = String::new();
        src.read_to_string(&mut src_data)?;

        let parser = Parser::new_ext(&src_data, options);
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);

        let mut out_file_name = path;
        out_file_name.set_extension("html");
        let out_path = guide_out_dir.join(out_file_name.file_name().unwrap());

        let mut out = fs::File::create(out_path)?;
        out.write_all("{% extends \"index.html\" %}\n\n".as_bytes())?;
        out.write_all(html.as_bytes())?;
    }

    println!("cargo:rerun-if-changed={}", guide_in_dir.to_str().unwrap());
    Ok(())
}
