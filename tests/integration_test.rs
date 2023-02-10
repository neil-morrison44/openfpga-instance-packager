use serde_json::json;
use std::{
    fs::{self, create_dir_all, File},
    path::PathBuf,
};
use tempfile::tempdir;

fn make_fake_files(files: Vec<&str>) -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    for file in files {
        let full_path = path.join(file);
        create_dir_all(&full_path.parent().unwrap()).unwrap();
        File::create(full_path).unwrap();
    }

    temp_dir
}

#[test]
fn test_build_some_nested() {
    let temp_dir = make_fake_files(vec![
        "Assets/platform_name/common/game_a/game_a (1).bin",
        "Assets/platform_name/common/game_a/game_a.cue",
        "Assets/platform_name/common/nested/game_b/game_b (1).bin",
        "Assets/platform_name/common/nested/game_b/game_b.cue",
        "Cores/core_name/instance-packager.json",
    ]);
    let temp_path = temp_dir.path();

    let instance_package_json = r#"
        {
            "output":"Assets/platform_name/core_name",
            "platform_id": "platform_name",
            "slot_limit": {
               "count": 28,
               "message": "oh no - too many"
            },
            "data_slots":[
               {
                  "id":100,
                  "filename":"*.cue",
                  "sort":"single",
                  "required":true,
                  "as_filename":true
               },
               {
                  "id":101,
                  "filename":"*.bin",
                  "sort":"ascending",
                  "required":true
               }
            ]
         }
    "#;

    fs::write(
        temp_path.join("Cores/core_name/instance-packager.json"),
        instance_package_json,
    )
    .unwrap();

    instance_packager::build_jsons_for_core(
        &temp_path.to_path_buf(),
        "core_name",
        true,
        |_file_name| {},
        |_file_name, _message| {},
    )
    .unwrap();

    let data =
        fs::read_to_string(temp_path.join("Assets/platform_name/core_name/game_a.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&data).unwrap();

    assert_eq!(
        &json,
        &json!({
            "instance": {
              "data_path": "game_a/",
              "data_slots": [
                {"filename": "game_a.cue", "id": 100},
                {"filename": "game_a (1).bin", "id": 101}
              ],
              "magic": "APF_VER_1",
              "memory_writes": []
            }
        })
    );

    let data =
        fs::read_to_string(temp_path.join("Assets/platform_name/core_name/nested/game_b.json"))
            .unwrap();
    let json: serde_json::Value = serde_json::from_str(&data).unwrap();

    assert_eq!(
        &json,
        &json!({
            "instance": {
              "data_path": "nested/game_b/",
              "data_slots": [
                {"filename": "game_b.cue", "id": 100},
                {"filename": "game_b (1).bin", "id": 101}
              ],
              "magic": "APF_VER_1",
              "memory_writes": []
            }
        })
    );
}

#[test]
fn test_build_too_many_files() {
    let temp_dir = make_fake_files(vec![
        "Cores/core_name/instance-packager.json",
        "Assets/platform_name/common/game_a/game_a.cue",
        "Assets/platform_name/common/game_a/game_a (1).bin",
        "Assets/platform_name/common/game_a/game_a (2).bin",
        "Assets/platform_name/common/game_a/game_a (3).bin",
        "Assets/platform_name/common/game_a/game_a (4).bin",
        "Assets/platform_name/common/game_a/game_a (5).bin",
        "Assets/platform_name/common/game_a/game_a (6).bin",
        "Assets/platform_name/common/game_a/game_a (7).bin",
        "Assets/platform_name/common/game_a/game_a (8).bin",
        "Assets/platform_name/common/game_a/game_a (9).bin",
        "Assets/platform_name/common/game_a/game_a (10).bin",
        "Assets/platform_name/common/game_a/game_a (11).bin",
        "Assets/platform_name/common/game_a/game_a (12).bin",
    ]);
    let temp_path = temp_dir.path();

    let instance_package_json = r#"
        {
            "output":"Assets/platform_name/core_name",
            "platform_id": "platform_name",
            "slot_limit": {
               "count": 10,
               "message": "oh no - too many"
            },
            "data_slots":[
               {
                  "id":100,
                  "filename":"*.cue",
                  "sort":"single",
                  "required":true,
                  "as_filename":true
               },
               {
                  "id":101,
                  "filename":"*.bin",
                  "sort":"ascending",
                  "required":true
               }
            ]
         }
    "#;

    fs::write(
        temp_path.join("Cores/core_name/instance-packager.json"),
        instance_package_json,
    )
    .unwrap();

    instance_packager::build_jsons_for_core(
        &temp_path.to_path_buf(),
        "core_name",
        true,
        |_file_name| {},
        |file_name, message| {
            assert_eq!(
                file_name,
                PathBuf::from("Assets/platform_name/core_name")
                    .join("game_a.json")
                    .to_str()
                    .unwrap()
            );
            assert_eq!(message, "oh no - too many")
        },
    )
    .unwrap();

    let exists = temp_path
        .join("Assets/platform_name/core_name/game_a.json")
        .exists();

    assert!(!exists);
}
