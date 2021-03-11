use cargo::core::Workspace;
use cargo::ops::compile;
use cargo::ops::CompileOptions;
use cargo::util::config::Config;

use cargo::core::compiler::CompileMode;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

fn call_dynamic() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("target/debug/libcrate0.dylib")?;
        let func: libloading::Symbol<fn()> = lib.get(b"external_function")?;
        Ok(func())
    }
}

fn main() {
    // cargo settings
    let config = Config::default().expect("failed to create default compiler config");
    let current_path = std::env::current_dir().expect("failed to get current path");
    let target_crate_path = current_path.join("crate0");
    let target_manifest_path = target_crate_path.join("Cargo.toml");
    let workspace =
        Workspace::new(&target_manifest_path, &config).expect("failed to create workspace");
    let compile_options =
        CompileOptions::new(&config, CompileMode::Build).expect("failed to create compile_options");
    let compiler = move || compile(&workspace, &compile_options);

    // watcher settings
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, Duration::from_millis(16)).expect("failed to create watcher");
    watcher
        .watch(&target_crate_path, RecursiveMode::Recursive)
        .expect("failed to start watching");

    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(path) => match path.extension() {
                    Some(ext) if ext == "rs" => {
                        println!("compiling...");
                        if compiler().is_ok() {
                            call_dynamic().expect("failed to run call_dynamic");
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Err(e) => {
                println!("watch error: {:?}", e);
            }
        }
    }
}
