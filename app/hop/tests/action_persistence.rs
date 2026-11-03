use hop_lib::commands::{delete_action, delete_group, save_action, save_group};
use uuid::Uuid;

/// Exercises save/edit/delete against the real config.json in this
/// machine's Hop config dir, same as group_persistence.rs.
#[test]
fn save_edit_and_delete_an_action_round_trip() {
    let id = Uuid::new_v4();
    let folder_a = Uuid::new_v4();
    let folder_b = Uuid::new_v4();
    let config = save_action(Some(id), "start".to_string(), "docker compose up".to_string(), vec![folder_a])
        .expect("save_action failed");
    let saved = config.actions.iter().find(|a| a.id == id).expect("saved action missing");
    assert_eq!(saved.name, "start");
    assert_eq!(saved.command, "docker compose up");
    assert_eq!(saved.group_ids, vec![folder_a]);

    let config = save_action(Some(id), "up".to_string(), "docker compose up -d".to_string(), vec![folder_a, folder_b])
        .expect("save_action (edit) failed");
    let edited: Vec<_> = config.actions.iter().filter(|a| a.id == id).collect();
    assert_eq!(edited.len(), 1, "editing must replace the action, not add a second one");
    assert_eq!(edited[0].name, "up");
    assert_eq!(edited[0].group_ids, vec![folder_a, folder_b]);

    let config = delete_action(id).expect("delete_action failed");
    assert!(config.actions.iter().all(|a| a.id != id), "action still present after delete");
}

/// Deleting a folder drops it from every action's folder list, but keeps
/// the action itself (it may still show in its other folders).
#[test]
fn deleting_a_folder_removes_it_from_actions_without_deleting_them() {
    let group_id = Uuid::new_v4();
    let other_group_id = Uuid::new_v4();
    save_group(Some(group_id), "temp folder for action test".to_string(), None).expect("save_group failed");

    let action_id = Uuid::new_v4();
    save_action(Some(action_id), "logs".to_string(), "docker compose logs -f".to_string(), vec![group_id, other_group_id])
        .expect("save_action failed");

    let config = delete_group(group_id).expect("delete_group failed");
    let action = config.actions.iter().find(|a| a.id == action_id).expect("action vanished after folder delete");
    assert_eq!(action.group_ids, vec![other_group_id]);

    let config = delete_action(action_id).expect("cleanup delete_action failed");
    assert!(config.actions.iter().all(|a| a.id != action_id));
}
