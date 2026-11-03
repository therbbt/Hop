use hop_lib::commands::{delete_action, save_action};
use uuid::Uuid;

/// Exercises save/edit/delete against the real config.json in this
/// machine's Hop config dir, same as group_persistence.rs.
#[test]
fn save_edit_and_delete_an_action_round_trip() {
    let id = Uuid::new_v4();
    let config = save_action(Some(id), "start".to_string(), "docker compose up".to_string(), vec!["~/app".to_string()])
        .expect("save_action failed");
    let saved = config.actions.iter().find(|a| a.id == id).expect("saved action missing");
    assert_eq!(saved.name, "start");
    assert_eq!(saved.command, "docker compose up");
    assert_eq!(saved.paths, vec!["~/app"]);

    let paths = vec!["~/app".to_string(), "/srv/app".to_string()];
    let config = save_action(Some(id), "up".to_string(), "docker compose up -d".to_string(), paths.clone())
        .expect("save_action (edit) failed");
    let edited: Vec<_> = config.actions.iter().filter(|a| a.id == id).collect();
    assert_eq!(edited.len(), 1, "editing must replace the action, not add a second one");
    assert_eq!(edited[0].name, "up");
    assert_eq!(edited[0].paths, paths);

    let config = delete_action(id).expect("delete_action failed");
    assert!(config.actions.iter().all(|a| a.id != id), "action still present after delete");
}
