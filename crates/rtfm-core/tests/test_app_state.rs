use rtfm_core::app_state::AppState;
use rtfm_core::clipboard::ClipboardMode;
use rtfm_core::task_manager::TaskKind;
use std::fs;
use tempdir::TempDir;

#[test]
fn test_new_app_state() {
    let app_state = AppState::new();
    assert_eq!(app_state.tabs.len(), 1);
    assert_eq!(app_state.active_tab_index, 0);
}

#[tokio::test]
async fn test_clipboard_yank_and_paste() {
    let tmp_dir = TempDir::new("test").unwrap();
    let file_path = tmp_dir.path().join("file.txt");
    fs::write(&file_path, "hello").unwrap();

    let mut app_state = AppState::new();
    app_state.get_active_tab_mut().current_dir = tmp_dir.path().to_path_buf();
    app_state.get_active_tab_mut().update_entries();

    // Yank the file
    app_state.get_active_tab_mut().cursor = 0; // Assuming the file is the first entry
    app_state.yank_selection();

    assert_eq!(app_state.clipboard.paths.len(), 1);
    assert_eq!(app_state.clipboard.paths[0], file_path);
    assert_eq!(app_state.clipboard.mode, Some(ClipboardMode::Copy));

    // "Paste" it
    app_state.paste();
    let tasks = app_state.task_manager.get_tasks();
    assert_eq!(tasks.len(), 1);
    match &tasks[0].kind {
        TaskKind::Copy { src, dest } => {
            assert_eq!(src, &file_path);
            assert_eq!(dest, &tmp_dir.path().join("file.txt"));
        }
        _ => panic!("Wrong task kind"),
    }
}

#[test]
fn test_new_tab() {
    let mut app_state = AppState::new();
    app_state.new_tab();
    assert_eq!(app_state.tabs.len(), 2);
    assert_eq!(app_state.active_tab_index, 1);
}

#[test]
fn test_close_tab() {
    let mut app_state = AppState::new();
    app_state.new_tab();
    app_state.close_tab();
    assert_eq!(app_state.tabs.len(), 1);
    assert_eq!(app_state.active_tab_index, 0);
}
