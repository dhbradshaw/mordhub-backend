use std::{collections::HashMap, fs, sync::RwLock};

lazy_static::lazy_static! {
    static ref FILES: FileCache<'static> = {
        FileCache::new()
            .file("static/404.html")
    };
}

struct FileCache<'a> {
    pub files: RwLock<HashMap<&'a str, String>>,
}

impl<'a> FileCache<'a> {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    pub fn file(self, path: &'a str) -> Self {
        match fs::read_to_string(path) {
            Ok(data) => {
                self.files.write().unwrap().insert(path, data);
            }
            Err(e) => panic!("no file found at location {}: {}", path, e),
        }

        self
    }
}

pub fn read(path: &str) -> String {
    match FILES.files.read().unwrap().get(path) {
        Some(data) => data.clone(),
        None => {
            error!("no file found in cache named '{}'", path);
            "internal error".to_owned()
        }
    }
}
