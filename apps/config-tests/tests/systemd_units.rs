//! The user units' .wants links (config/systemd): stow refuses absolute links in a package, and
//! ~/.config/systemd is a real folder since agent-cockpit and vitals link their units into it too.

mod common;
use common::sandbox::*;

use std::fs;
use std::path::Path;

#[test]
fn every_wants_link_is_relative_and_names_a_unit_of_this_package_or_of_the_system() {
    let user = repo().join("config/systemd/.config/systemd/user");
    for wants in fs::read_dir(&user).unwrap().flatten().filter(|entry| entry.file_name().to_string_lossy().ends_with(".wants")) {
        for link in fs::read_dir(wants.path()).unwrap().flatten() {
            let name = link.file_name().to_string_lossy().into_owned();
            let target = fs::read_link(link.path()).unwrap();
            assert!(target.is_relative(), "{name} -> {}: stow refuses absolute links", target.display());
            let packaged = target == Path::new("..").join(&name) && user.join(&name).is_file();
            let system = target.to_string_lossy().ends_with(&format!("/usr/lib/systemd/user/{name}"));
            assert!(packaged || system, "{name} -> {}: neither ../{name} in this package nor a packaged unit", target.display());
        }
    }
}
