use std::time::{Duration, Instant};

use dam_application::ConfigError;

use super::project_support::ProjectFolder;

fn refusal_of(text: &str) -> (ProjectFolder, ConfigError) {
    let folder = ProjectFolder::with(text);
    let refusal = folder.read().unwrap_err();
    (folder, refusal)
}

fn refused_naming_only_the_file(text: &str) {
    let (folder, refusal) = refusal_of(text);
    match refusal {
        ConfigError::StoppedLoading { path, reason, .. } => {
            assert_eq!(path, folder.file(), "{text}");
            assert_eq!(reason, None, "{text}");
        }
        other => panic!("{text}: {other:?}"),
    }
}

#[test]
fn every_escape_to_the_system_is_refused() {
    for text in [
        "os.execute('touch MARK') return {}",
        "return { path = os.getenv('HOME') }",
        "io.open('MARK', 'w') return {}",
        "return { path = io.read('a') }",
        "require('os') return {}",
        "package.loadlib('x', 'y') return {}",
        "dofile('secrets.lua') return {}",
        "loadfile('secrets.lua')() return {}",
        "load('return 1')() return {}",
        "loadstring('return 1')() return {}",
        "debug.getregistry() return {}",
        "string.dump(function() end) return {}",
        "getmetatable('').__index.dump(print) return {}",
        "collectgarbage('count') return {}",
        "print('hello') return {}",
        "coroutine.wrap(function() end)() return {}",
        "pcall(error) return {}",
        "error('\\27[31mred') return {}",
        "setmetatable({}, { __gc = function() while true do end end }) return {}",
        "string.find(('a'):rep(30), '.-.-.-.-.-.-b') return {}",
    ] {
        refused_naming_only_the_file(text);
    }
}

#[test]
fn no_escape_touches_the_folder() {
    let (folder, _) = refusal_of("os.execute('touch MARK') io.open('MARK', 'w') return {}");
    assert!(!folder.holds("MARK"));
}

#[test]
fn bytecode_is_refused() {
    let (folder, refusal) = refusal_of("\x1bLua\x54\x00");
    assert!(matches!(refusal, ConfigError::StoppedLoading { path, .. } if path == folder.file()));
}

#[test]
fn an_endless_loop_is_stopped() {
    let started = Instant::now();
    refused_naming_only_the_file("while true do end");
    refused_naming_only_the_file("local function f() return f() end return f()");
    refused_naming_only_the_file("local function f() return 1 + f() end return f()");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn a_huge_allocation_is_stopped() {
    refused_naming_only_the_file("local s = 'x' for i = 1, 26 do s = s .. s end return {}");
    refused_naming_only_the_file("local t = {} for i = 1, 1e6 do t[i] = i end return {}");
}

#[test]
fn deep_nesting_is_refused() {
    let (folder, refusal) = refusal_of(&format!("return {}", "{".repeat(100_000)));
    assert!(matches!(refusal, ConfigError::StoppedLoading { path, .. } if path == folder.file()));
}
