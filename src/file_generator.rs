use std::path::Path;
use std::{fs, io::Write};

use crate::descriptor::{DescriptorManager, TS};
use crate::utils::remove_ext;

const PREFIX: &'static str = r#"// DO NOT EDIT. CODE GENERATED BY gents."#;

/// Members from a FileGroup will:
/// - generate the .ts files in the same directory
/// - share their dependencies
/// - occur overwriting if some of them share the same file name
pub struct FileGroup {
    manager: DescriptorManager,
}

impl FileGroup {
    pub fn new() -> Self {
        FileGroup {
            manager: DescriptorManager::default(),
        }
    }

    /// Add a TS member into this FileGroup.
    pub fn add<T: TS>(&mut self) {
        T::_register(&mut self.manager);
    }

    pub fn gen_files(self, dir: &str, index_file: bool) {
        let mut data = self.manager.gen_data();
        if index_file {
            let mut exports: Vec<String> = vec![];
            data.iter().for_each(|(file_name, _)| {
                let s = format!(r#"export * from './{}'"#, remove_ext(&file_name));
                exports.push(s);
            });
            exports.sort();
            exports.push(String::from(""));
            let content = exports.join("\n");
            data.push((String::from("index.ts"), content));
        }
        data.into_iter().for_each(|(file_name, content)| {
            let mut file_path = Path::new(dir).to_path_buf();
            file_path.push(file_name);
            if let Some(p) = file_path.parent() {
                fs::create_dir_all(p).expect("create dir failed");
            }
            let mut f = fs::File::create(file_path).expect("create file error");
            f.write_all(PREFIX.as_ref()).expect("write prefix error");
            f.write("\n".as_ref()).expect("");
            f.write_all(content.as_ref()).expect("write content error");
        });
    }
}
