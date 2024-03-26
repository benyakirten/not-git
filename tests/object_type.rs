use std::fs;

use not_git::objects::ObjectType;

mod common;

#[test]
fn from_entry() {
    let path = common::TestPath::new();

    let sub_dir = path.join(&"B_sub_dir");
    fs::create_dir(&sub_dir).unwrap();

    let sub_file = path.join(&"A_sub_file");
    fs::File::create(sub_file).unwrap();

    let mut dir = path.0.read_dir().unwrap();

    let object_type = ObjectType::from_entry(&dir.next().unwrap().unwrap()).unwrap();
    assert_eq!(object_type, ObjectType::Blob);

    let object_type = ObjectType::from_entry(&dir.next().unwrap().unwrap()).unwrap();
    assert_eq!(object_type, ObjectType::Tree);
}
