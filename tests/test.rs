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
            content,
            r#"import {Person} from './person'

export interface Group {
    name: string
    capacity: number
    members: readonly Person[]
    leader: Person | null
}
"#
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
    | {cat: string}
    | {dog: string}
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
        );
    }
}
