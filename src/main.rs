use std::env;
use std::path::PathBuf;

mod git_actions;
mod hg_actions;
mod revision_shortcut;
mod select;
mod tui;
mod version_control_actions;

use git_actions::GitActions;
use hg_actions::HgActions;
use revision_shortcut::RevisionShortcut;

fn main() {
	ctrlc::set_handler(move || {}).unwrap();

	let current_dir_path = env::current_dir().unwrap();
	let current_dir = current_dir_path.to_str().unwrap();

	let revision_shortcut = RevisionShortcut::default();

	if subdir_exists(&current_dir_path, ".git") {
		let actions = GitActions {
			current_dir: current_dir.into(),
			revision_shortcut: revision_shortcut,
		};
		tui::show_tui(vec![Box::new(actions)]);
	} else if subdir_exists(&current_dir_path, ".hg") {
		let actions = HgActions {
			current_dir: current_dir.into(),
			revision_shortcut: revision_shortcut,
		};
		tui::show_tui(vec![Box::new(actions)]);
	} else {
		println!("no repository found");
	}
}

fn subdir_exists(basedir: &PathBuf, subdir: &str) -> bool {
	let mut path = basedir.clone();
	path.push(subdir);
	path.exists()
}
