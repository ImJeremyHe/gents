use gents::FileGroup;
use gents_derives::TS;
use serde::{Deserialize, Serialize};
use std::{fs, process::Command};

const TS_TESTS_DIR: &str = "src/serde_test/data";
const CHECK_PATTERN: &str = r#"
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

import type { {type_name} } from '../data/{file_name}'

const jsonFile = path.resolve(__dirname, '../data/{file_name}.json')
const jsonData = JSON.parse(fs.readFileSync(jsonFile, 'utf8'))

const typedData: {type_name} = jsonData

console.log('✅ JSON successfully matches the {file_name} TypeScript interface')
console.log('Example:', typedData)
"#;

#[derive(Serialize, Deserialize, TS)]
#[ts(file_name = "user.ts")]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[test]
fn test_ts_rust_json_compatibility() {
    let file_name = "user";
    let type_name = "User";
    let mut file_group = FileGroup::new();
    file_group.add::<User>();
    file_group.gen_files(TS_TESTS_DIR, false);

    let user = User {
        id: 42,
        name: "Alice".to_string(),
    };
    let json = serde_json::to_string(&user).unwrap();
    fs::write(format!("{}/{}.json", TS_TESTS_DIR, file_name), &json).unwrap();

    let ts_file_content = CHECK_PATTERN
        .replace("{file_name}", file_name)
        .replace("{type_name}", type_name);
    fs::write("src/serde_test/ts/check.ts", ts_file_content).unwrap();

    let path = fs::canonicalize("src/serde_test/ts").unwrap();

    let status = Command::new("npx")
        .args(["tsx", "check.ts"])
        .current_dir(path)
        .status()
        .unwrap();
    assert!(status.success(), "TypeScript failed to check types");

    let gen_json = fs::read_to_string(format!("{}/{}.json", TS_TESTS_DIR, file_name)).unwrap();
    let user_back: User = serde_json::from_str(&gen_json).unwrap();

    assert_eq!(user_back.id, 42);
    assert_eq!(user_back.name, "Alice");
}
