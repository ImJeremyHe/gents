#[allow(dead_code)]
#[cfg(test)]
mod tests;

#[allow(dead_code)]
#[cfg(test)]
mod serde_test;

#[derive(Debug, Clone, gents_derives::TS)]
#[ts(file_name = "a.ts", rename_all = "camelCase")]
pub enum VerticalAlignment {
    Center,
    Top,
    Bottom,
    Justify,
    Distributed,
}

#[test]
fn test_unit_enum_serde() {
    let vertical_alignment = VerticalAlignment::Center;
    let json = serde_json::to_string(&vertical_alignment).unwrap();
    assert_eq!(json, "\"center\"");
    let vertical_alignment = serde_json::from_str::<VerticalAlignment>("\"center\"").unwrap();
    assert!(matches!(vertical_alignment, VerticalAlignment::Center));
}

#[derive(Debug, Clone, gents_derives::TS)]
#[ts(file_name = "b.ts", rename_all = "camelCase")]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[test]
fn test_struct_serde() {
    let user = User {
        id: 1,
        name: "test".to_string(),
    };
    let json = serde_json::to_string(&user).unwrap();
    assert_eq!(json, "{\"id\":1,\"name\":\"test\"}");
    let user = serde_json::from_str::<User>("{\"id\":1,\"name\":\"test\"}").unwrap();
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "test");
}

#[derive(Debug, Clone, gents_derives::TS)]
#[ts(file_name = "c.ts", rename_all = "camelCase", tag = "type")]
pub enum TestEnum {
    Center,
    Top,
    Bottom,
    Justify,
    Distributed,
    Unknown(User),
}

#[test]
fn test_struct_tagged_enum_serde() {
    let center = TestEnum::Center;
    let json = serde_json::to_string(&center).unwrap();
    assert_eq!(json, "\"center\"");
    let center = serde_json::from_str::<TestEnum>("\"center\"").unwrap();
    assert!(matches!(center, TestEnum::Center));

    let user = TestEnum::Unknown(User {
        id: 1,
        name: "test".to_string(),
    });
    let json = serde_json::to_string(&user).unwrap();
    assert_eq!(
        json,
        "{\"type\":\"unknown\",\"value\":{\"id\":1,\"name\":\"test\"}}"
    );
    let user = serde_json::from_str::<TestEnum>(
        "{\"type\":\"unknown\",\"value\":{\"id\":1,\"name\":\"test\"}}",
    )
    .unwrap();
    match user {
        TestEnum::Unknown(user) => {
            assert_eq!(user.id, 1);
            assert_eq!(user.name, "test");
        }
        _ => panic!(),
    }
}
