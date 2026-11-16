use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{ProjectFolders, find_project};

struct Holding(BTreeSet<PathBuf>);

impl Holding {
    fn files_in(folders: &[&str]) -> Holding {
        Holding(folders.iter().map(PathBuf::from).collect())
    }
}

impl ProjectFolders for Holding {
    fn holds_project_file(&self, folder: &Path) -> bool {
        self.0.contains(folder)
    }
}

const CONFIG: &str = "/home/someone/.config/dam";

fn found(start: &str, holding: &Holding) -> Option<PathBuf> {
    find_project(Path::new(start), Path::new(CONFIG), holding)
}

#[test]
fn the_project_is_the_nearest_folder_holding_the_file() {
    let holding = Holding::files_in(&["/work", "/work/app"]);
    assert_eq!(
        found("/work/app/src/deep", &holding),
        Some("/work/app".into())
    );
    assert_eq!(found("/work/app", &holding), Some("/work/app".into()));
    assert_eq!(found("/work/lib", &holding), Some("/work".into()));
}

#[test]
fn a_folder_with_no_project_above_it_has_none() {
    assert_eq!(found("/work/app", &Holding::files_in(&[])), None);
}

#[test]
fn the_personal_config_folder_is_never_a_project() {
    let holding = Holding::files_in(&[CONFIG]);
    assert_eq!(found(CONFIG, &holding), None);
    assert_eq!(found(&format!("{CONFIG}/hooks"), &holding), None);
}
