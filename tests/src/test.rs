use gents_derives::TS;

#[derive(TS)]
#[ts(file_name = "person.ts", rename_all = "camelCase")]
pub struct Person {
    pub age: u16,
    pub en_name: String,
}

#[derive(TS)]
#[ts(file_name = "group.ts", rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    pub capacity: u16,
    pub members: Vec<Person>,
    pub leader: Option<Person>,
}

#[derive(TS)]
#[ts(file_name = "gender.ts")]
pub enum Gender {
    Male,
    Female,
    #[ts(rename = "null")]
    Unknown,
}

#[derive(TS)]
#[ts(file_name = "pet.ts", rename_all = "camelCase")]
pub enum Pet {
    Cat(String),
    Dog(String),
    #[ts(rename = "None")]
    None,
}

#[derive(TS)]
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
    use gents_derives::gents_header;

    #[test]
    fn gen_skip_test() {
        let mut manager = DescriptorManager::default();
        TestSkip::_register(&mut manager);
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
        Person::_register(&mut manager);
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
        Group::_register(&mut manager);
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
        Gender::_register(&mut manager);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "gender.ts");
        assert_eq!(
            content.trim(),
            r#"export type Gender =
    | 'Male'
    | 'Female'
    | 'null'"#
        );
    }

    #[test]
    fn gen_data_pet_test() {
        let mut manager = DescriptorManager::default();
        Pet::_register(&mut manager);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "pet.ts");
        assert_eq!(
            content.trim(),
            r#"export type Pet =
    | { cat: string }
    | { dog: string }
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
        #[derive(TS)]
        #[ts(file_name = "struct_with_comments.ts", rename_all = "camelCase")]
        pub struct StructWithComments {
            /// field comment1
            /// field comment2
            pub field_with_comment: u32,
        }

        let mut manager = DescriptorManager::default();
        StructWithComments::_register(&mut manager);
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
        #[derive(TS)]
        #[ts(file_name = "file.ts", rename_all = "camelCase")]
        pub struct File {
            pub data: Vec<u8>,
        }

        let mut manager = DescriptorManager::default();
        File::_register(&mut manager);
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
        #[derive(TS)]
        #[ts(file_name = "a.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct A {
            pub f1: u8,
        }

        let mut manager = DescriptorManager::default();
        A::_register(&mut manager);
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
        #[derive(TS)]
        #[ts(file_name = "a.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct Variant {
            pub f1: u8,
            pub f2: String,
        }

        #[derive(TS)]
        #[ts(file_name = "a.ts", rename_all = "camelCase", tag = "type")]
        pub enum TaggedEnum {
            Variant(Variant),
        }
        let mut manager = DescriptorManager::default();
        TaggedEnum::_register(&mut manager);
        let (file_name, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(file_name, "a.ts");
        assert_eq!(
            content.trim(),
            r#"
export interface Variant {
    type: 'variant'
    f1: number
    f2: string
}

export class VariantBuilder {
    private _type = 'variant'
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
        return { type: this._type, f1: this._f1, f2: this._f2 }
    }
}"#
            .trim()
        );
    }

    #[test]
    fn test_enum_tag_and_builder_2() {
        #[derive(TS)]
        #[ts(builder, file_name = "a.ts", rename_all = "camelCase")]
        pub struct V1 {
            pub f1: u8,
            pub f2: String,
        }

        #[derive(TS)]
        #[ts(file_name = "b.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct V2 {
            pub f3: u8,
        }

        #[derive(TS)]
        #[ts(file_name = "c.ts", rename_all = "camelCase", tag = "type")]
        pub enum TaggedEnum {
            #[ts(tag_value = "tag1")]
            V1(V1),
            #[ts(tag_value = "tag2")]
            V2(V2),
            #[ts(tag_value = "tag3")]
            V3(V1),
        }

        let mut manager = DescriptorManager::default();
        TaggedEnum::_register(&mut manager);
        manager.gen_data().into_iter().for_each(|(f, c)| {
            if f == "a.ts" {
                assert_eq!(
                    c.trim(),
                    r#"
export interface V1 {
    type: 'tag1' | 'tag3'
    f1: number
    f2: string
}

export class V1Builder {
    private _type!: 'tag1' | 'tag3'
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
    public type(v: 'tag1' | 'tag3') {
        this._type = v
        return this
    }
    public build() {
        if (this._f1 === undefined) throw new Error('missing f1')
        if (this._f2 === undefined) throw new Error('missing f2')
        return { type: this._type, f1: this._f1, f2: this._f2 }
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
    type: 'tag2'
    f3: number
}

export class V2Builder {
    private _type = 'tag2'
    private _f3!: number
    public f3(value: number) {
        this._f3 = value
        return this
    }

    public build() {
        if (this._f3 === undefined) throw new Error('missing f3')
        return { type: this._type, f3: this._f3 }
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
    | V1
    | V2
    | V1"#
                        .trim()
                );
            }
        });
    }

    #[test]
    fn test_multiple_tags() {
        #[derive(TS)]
        #[ts(file_name = "a.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct V1 {
            pub f1: u8,
            pub f2: String,
        }

        #[derive(TS)]
        #[ts(file_name = "b.ts", rename_all = "camelCase")]
        #[ts(builder)]
        pub struct V2 {
            pub f3: u8,
        }

        #[derive(TS)]
        #[ts(file_name = "a.ts", tag = "type")]
        pub enum A {
            V1(V1),
            V2(V2),
            V3(V1),
        }

        #[derive(TS)]
        #[ts(file_name = "b.ts", tag = "type2")]
        pub enum B {
            V1(V1),
            V2(V2),
            V3(V1),
        }

        let mut manager = DescriptorManager::default();
        A::_register(&mut manager);
        B::_register(&mut manager);
        let data = manager.gen_data();
        assert_eq!(data.len(), 4);
    }

    #[test]
    fn test_result() {
        #[gents_header(file_name = "test_struct.ts")]
        pub struct TestStruct {
            pub f1: u8,
            pub f2: Result<String, u16>,
        }
        let mut manager = DescriptorManager::default();
        TestStruct::_register(&mut manager);
        let (_, content) = manager.gen_data().into_iter().next().unwrap();
        assert_eq!(
            content.trim(),
            r#"export interface TestStruct {
    f1: number
    f2: string | number
}"#
        );
    }

    #[test]
    fn test_gents_for_wasm() {
        #[gents_header(file_name = "test_struct.ts")]
        pub struct TestStruct {
            pub f1: u8,
            pub f2: String,
        }
    }
}
