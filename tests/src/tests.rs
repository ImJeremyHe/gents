use gents_derives::TS;

#[derive(TS, Clone)]
#[ts(file_name = "person.ts", rename_all = "camelCase")]
pub struct Person {
    pub age: u16,
    pub en_name: String,
}

#[derive(TS, Clone)]
#[ts(file_name = "group.ts", rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    pub capacity: u16,
    pub members: Vec<Person>,
    pub leader: Option<Person>,
}

#[derive(TS, Clone)]
#[ts(file_name = "gender.ts")]
pub enum Gender {
    Male,
    Female,
    #[ts(rename = "null")]
    Unknown,
}

#[derive(TS, Clone)]
#[ts(file_name = "pet.ts", rename_all = "camelCase")]
#[ts(tag = "type")]
pub enum Pet {
    Cat(String),
    Dog(String),
    #[ts(rename = "None")]
    None,
}

#[derive(TS, Clone)]
#[ts(file_name = "skip.ts", rename_all = "camelCase")]
pub struct TestSkip {
    pub f1: u16,
    #[ts(skip = true)]
    pub f2: u32,
    pub f3: u64,
}

#[cfg(test)]
mod tests {

    use super::*;
    use gents::*;

    #[test]
    fn gen_skip_test() {
        let mut manager = DescriptorManager::default();
        TestSkip::_register(&mut manager, true);
        let (_, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(
            content.trim(),
            r#"export interface TestSkip {
    f1: number
    f3: number
}"#
        )
    }

    #[test]
    fn gen_data_person_test() {
        let mut manager = DescriptorManager::default();
        Person::_register(&mut manager, true);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "person.ts");
        assert_eq!(
            content.trim(),
            r#"export interface Person {
    age: number
    enName: string
}"#
        )
    }

    #[test]
    fn gen_data_group_test() {
        let mut manager = DescriptorManager::default();
        Group::_register(&mut manager, true);
        let (file_name, content) = manager.gen_data().into_iter().last().unwrap();
        assert_eq!(file_name, "group.ts");
        assert_eq!(
            content.trim(),
            r#"
import { Person } from './person'

export interface Group {
    name: string
    capacity: number
    members: readonly Person[]
    leader?: Person
}
"#
            .trim()
        );
    }

    #[test]
    fn gen_data_gender_test() {
        let mut manager = DescriptorManager::default();
        Gender::_register(&mut manager, true);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "gender.ts");
        assert_eq!(
            content.trim(),
            r#"export type Gender =
    | 'male'
    | 'female'
    | 'null'"#
        );
    }

    #[test]
    fn gen_data_pet_test() {
        let mut manager = DescriptorManager::default();
        Pet::_register(&mut manager, true);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "pet.ts");
        assert_eq!(
            content.trim(),
            r#"export type Pet =
    | { type: 'cat'; value: string }
    | { type: 'dog'; value: string }
    | 'None'"#
        );
    }

    #[test]
    fn gen_with_comments_test() {
        /// This is a doc comment.
        /// Another Comment
        /**
        Block Comment
        */
        #[derive(TS, Clone)]
        #[ts(file_name = "struct_with_comments.ts", rename_all = "camelCase")]
        pub struct StructWithComments {
            /// field comment1
            /// field comment2
            pub field_with_comment: u32,
        }

        let mut manager = DescriptorManager::default();
        StructWithComments::_register(&mut manager, true);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "struct_with_comments.ts");
        assert_eq!(
            content.trim(),
            r#"// This is a doc comment.
// Another Comment
// Block Comment
export interface StructWithComments {
    // field comment1
    // field comment2
    fieldWithComment: number
}"#
        );
    }

    #[test]
    fn test_uint8array() {
        #[derive(TS, Clone)]
        #[ts(file_name = "file.ts", rename_all = "camelCase")]
        pub struct File {
            pub data: Vec<u8>,
        }

        let mut manager = DescriptorManager::default();
        File::_register(&mut manager, true);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "file.ts");
        assert_eq!(
            content.trim(),
            r#"export interface File {
    data: Uint8Array
}"#
            .trim()
        );
    }

    #[test]
    fn test_builder() {
        #[derive(TS, Clone)]
        #[ts(file_name = "a.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct A {
            pub f1: u8,
        }

        let mut manager = DescriptorManager::default();
        A::_register(&mut manager, true);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "a.ts");
        assert_eq!(
            content.trim(),
            r#"
export interface A {
    f1: number
}

export class ABuilder {
    private _f1!: number
    public f1(value: number) {
        this._f1 = value
        return this
    }
    public build() {
        if (this._f1 === undefined) throw new Error('missing f1')
        return { f1: this._f1 }
    }
}
"#
            .trim(),
        );
    }

    #[test]
    fn test_enum_tag_and_builder() {
        #[derive(TS, Clone)]
        #[ts(file_name = "a.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct Variant {
            pub f1: u8,
            pub f2: String,
        }

        #[derive(TS, Clone)]
        #[ts(file_name = "a.ts", rename_all = "camelCase", tag = "type")]
        pub enum TaggedEnum {
            Variant(Variant),
        }
        let mut manager = DescriptorManager::default();
        TaggedEnum::_register(&mut manager, true);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "a.ts");
        assert_eq!(
            content.trim(),
            r#"
export interface Variant {
    f1: number
    f2: string
}

export class VariantBuilder {
    private _f1!: number
    private _f2!: string
    public f1(value: number) {
        this._f1 = value
        return this
    }

    public f2(value: string) {
        this._f2 = value
        return this
    }
    public build() {
        if (this._f1 === undefined) throw new Error('missing f1')
        if (this._f2 === undefined) throw new Error('missing f2')
        return { f1: this._f1, f2: this._f2 }
    }
}"#
            .trim()
        );
    }

    #[test]
    fn test_enum_tag_and_builder_2() {
        #[derive(TS, Clone)]
        #[ts(builder, file_name = "a.ts", rename_all = "camelCase")]
        pub struct V1 {
            pub f1: u8,
            pub f2: String,
        }

        #[derive(TS, Clone)]
        #[ts(file_name = "b.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct V2 {
            pub f3: u8,
        }

        #[derive(TS, Clone)]
        #[ts(file_name = "c.ts", rename_all = "camelCase", tag = "type")]
        pub enum TaggedEnum {
            V1(V1),
            V2(V2),
            V3(V1),
        }

        let mut manager = DescriptorManager::default();
        TaggedEnum::_register(&mut manager, true);
        manager.gen_data().into_iter().for_each(|(f, c)| {
            if f == "a.ts" {
                assert_eq!(
                    c.trim(),
                    r#"
export interface V1 {
    f1: number
    f2: string
}

export class V1Builder {
    private _f1!: number
    private _f2!: string
    public f1(value: number) {
        this._f1 = value
        return this
    }

    public f2(value: string) {
        this._f2 = value
        return this
    }
    public build() {
        if (this._f1 === undefined) throw new Error('missing f1')
        if (this._f2 === undefined) throw new Error('missing f2')
        return { f1: this._f1, f2: this._f2 }
    }
}"#
                    .trim()
                );
            }
            if f == "b.ts" {
                assert_eq!(
                    c.trim(),
                    r#"
export interface V2 {
    f3: number
}

export class V2Builder {
    private _f3!: number
    public f3(value: number) {
        this._f3 = value
        return this
    }
    public build() {
        if (this._f3 === undefined) throw new Error('missing f3')
        return { f3: this._f3 }
    }
}"#
                    .trim()
                );
            }
            if f == "c.ts" {
                assert_eq!(
                    c.trim(),
                    r#"
import { V1 } from './a'
import { V2 } from './b'

export type TaggedEnum =
    | { type: 'v1'; value: V1 }
    | { type: 'v2'; value: V2 }
    | { type: 'v3'; value: V1 }"#
                        .trim()
                );
            }
        });
    }

    #[test]
    fn test_generic() {
        #[derive(TS, Clone)]
        #[ts(file_name = "a.ts", rename_all = "camelCase")]
        pub struct V1<T: TS> {
            pub f1: T,
        }

        let mut manager = DescriptorManager::default();
        V1::<String>::_register(&mut manager, true);
        let (_, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(
            content.trim(),
            r#"export interface V1<T> {
    f1: T
}"#
        );

        #[derive(TS, Clone)]
        #[ts(file_name = "b.ts", rename_all = "camelCase")]
        pub struct V2<T: TS> {
            pub f1: V1<T>,
            pub f2: T,
        }
        let mut manager = DescriptorManager::default();
        V2::<String>::_register(&mut manager, true);
        let (_, content) = manager.gen_data().into_iter().last().unwrap();
        assert_eq!(
            content.trim(),
            r#"import { V1 } from './a'

export interface V2<T> {
    f1: V1<T>
    f2: T
}"#
        );

        #[derive(TS, Clone)]
        #[ts(file_name = "c.ts", rename_all = "camelCase")]
        pub struct V3 {
            pub f1: V2<String>,
        }
        let mut manager = DescriptorManager::default();
        V3::_register(&mut manager, true);
        let (_, content) = manager.gen_data().into_iter().last().unwrap();
        assert_eq!(
            content.trim(),
            r#"import { V2 } from './b'

export interface V3 {
    f1: V2<string>
}"#,
        );
    }

    #[test]
    fn test_multiple_tags() {
        #[derive(TS, Clone)]
        #[ts(file_name = "a.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct V1 {
            pub f1: u8,
            pub f2: String,
        }

        #[derive(TS, Clone)]
        #[ts(file_name = "b.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct V2 {
            pub f3: u8,
        }

        #[derive(TS, Clone)]
        #[ts(file_name = "a.ts", tag = "type")]
        pub enum A {
            V1(V1),
            V2(V2),
            V3(V1),
        }

        #[derive(TS, Clone)]
        #[ts(file_name = "b.ts", tag = "type2")]
        pub enum B {
            V1(V1),
            V2(V2),
            V3(V1),
        }

        let mut manager = DescriptorManager::default();
        A::_register(&mut manager, true);
        B::_register(&mut manager, true);
        let data = manager.gen_data();
        assert_eq!(data.len(), 4);
    }
}

#[cfg(test)]
mod test_rpc_interface {
    use gents::*;
    use gents_derives::{Interface, TS};

    #[derive(TS, Clone)]
    #[ts(file_name = "get_cell_params.ts", rename_all = "camelCase")]
    pub struct GetCellParams {
        pub row: u32,
        pub col: u32,
    }

    #[derive(TS, Clone)]
    #[ts(file_name = "cell_info.ts", rename_all = "camelCase")]
    pub struct CellInfo {
        pub value: String,
    }

    #[derive(TS, Clone)]
    #[ts(file_name = "save_params.ts", rename_all = "camelCase")]
    pub struct SaveParams {
        pub path: String,
    }

    #[derive(TS, Clone)]
    #[ts(file_name = "save_result.ts", rename_all = "camelCase")]
    pub struct SaveResult {
        pub success: bool,
    }

    #[derive(TS, Clone)]
    #[ts(file_name = "error_message.ts", rename_all = "camelCase")]
    pub struct ErrorMessage {
        pub message: String,
    }

    // Without rename_all: keeps original snake_case names
    #[derive(Interface)]
    #[ts(file_name = "workbook_methods.ts")]
    pub struct WorkbookMethods {
        pub get_cell: fn(row_idx: u32, col_idx: u32) -> Result<CellInfo, ErrorMessage>,
        pub save_file: fn(file_path: String) -> Result<SaveResult, ErrorMessage>,
        pub delete: fn(id: u32) -> CellInfo,
        pub update: fn(id: u32, value: Option<String>) -> Result<(), ErrorMessage>,
    }

    // With rename_all = "camelCase": converts to camelCase
    #[derive(Interface)]
    #[ts(file_name = "workbook_methods_camel.ts", rename_all = "camelCase")]
    pub struct WorkbookMethodsCamel {
        pub get_cell: fn(row_idx: u32, col_idx: u32) -> Result<CellInfo, ErrorMessage>,
        pub save_file:
            fn(file_path: String, overwrite: Option<bool>) -> Result<SaveResult, ErrorMessage>,
    }

    #[test]
    fn test_rpc_interface_no_rename() {
        let mut manager = DescriptorManager::default();
        let desc = WorkbookMethods::__get_rpc_descriptor(&mut manager);
        manager.add_rpc_descriptor(desc);

        let data = manager.gen_data();
        let files: std::collections::HashMap<&str, &str> = data
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_str()))
            .collect();

        let content = files.get("workbook_methods.ts").unwrap();
        // Multiple params, Result return
        assert!(content.contains(
            "get_cell(row_idx: number, col_idx: number): Promise<CellInfo | ErrorMessage>;"
        ));
        // Single param, Result return
        assert!(
            content.contains("save_file(file_path: string): Promise<SaveResult | ErrorMessage>;")
        );
        // Non-Result return type
        assert!(content.contains("delete(id: number): Promise<CellInfo>;"));
        // Optional param + void ok type
        assert!(
            content.contains("update(id: number, value?: string): Promise<void | ErrorMessage>;")
        );
    }

    #[test]
    fn test_rpc_interface_with_rename_all() {
        let mut manager = DescriptorManager::default();
        let desc = WorkbookMethodsCamel::__get_rpc_descriptor(&mut manager);
        manager.add_rpc_descriptor(desc);

        let data = manager.gen_data();
        let files: std::collections::HashMap<&str, &str> = data
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_str()))
            .collect();

        let content = files.get("workbook_methods_camel.ts").unwrap();
        // camelCase + Result return
        assert!(content.contains(
            "getCell(rowIdx: number, colIdx: number): Promise<CellInfo | ErrorMessage>;"
        ));
        // camelCase + optional param
        assert!(content.contains(
            "saveFile(filePath: string, overwrite?: boolean): Promise<SaveResult | ErrorMessage>;"
        ));
    }
}

#[cfg(test)]
mod test_api {
    use gents::*;
    use gents_derives::{TS, ts_interface};

    #[derive(TS, Clone)]
    #[ts(file_name = "a.ts", rename_all = "camelCase")]
    pub struct V1 {
        pub f1: u8,
        pub f2: String,
    }

    /// API
    #[ts_interface(file_name = "v1_api.ts", ident = "V1Api")]
    impl V1 {
        pub fn f1(&self) -> u8 {
            self.f1
        }

        pub fn f2(&self) -> &str {
            &self.f2
        }

        /// set f1
        pub fn set_f1(&mut self, f1: u8) {
            self.f1 = f1;
        }

        pub fn set_f2(&mut self, f2: String) {
            self.f2 = f2;
        }
    }

    #[test]
    fn test_v1_api() {
        let mut manager = DescriptorManager::default();
        V1::_register(&mut manager, true);
        manager.add_api_descriptor(V1::__get_api_descriptor());
        let data = manager.gen_data();
        assert_eq!(data.len(), 2);
        let files: std::collections::HashMap<&str, &str> = data
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_str()))
            .collect();
        assert!(files.contains_key("a.ts"));
        assert!(files.contains_key("v1_api.ts"));

        assert_eq!(
            files.get("v1_api.ts").unwrap(),
            &"export interface V1Api {\n    f1(): number;\n    f2(): string;\n    // set f1\n    setF1(f1: number): void;\n    setF2(f2: string): void;\n}\n"
        );
    }
}
