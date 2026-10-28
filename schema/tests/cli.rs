use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use alkahest_schema::{MAX_JSON_BYTES, Snapshot};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        let number = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "alkahest-schema-cli-{}-{number}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn run(args: &[&OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_alkahest-schema"))
        .args(args)
        .output()
        .unwrap()
}

fn compare(baseline: &Path, current: &Path, format: &str) -> Output {
    run(&[
        OsStr::new("check"),
        baseline.as_os_str(),
        current.as_os_str(),
        OsStr::new("--format"),
        OsStr::new(format),
    ])
}

fn snapshot<const V: u32>() -> String {
    let mut snapshot = Snapshot::new();
    snapshot.add::<u8, V>("test.value").unwrap();
    snapshot.to_json().unwrap()
}

fn json(output: &Output) -> serde_json::Value {
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn passing_snapshot_and_version_increase_have_machine_readable_success() {
    let directory = Directory::new();
    let baseline_text = snapshot::<0>();
    let baseline = directory.write("baseline with spaces.json", &baseline_text);
    for current_text in [baseline_text.clone(), snapshot::<1>()] {
        let current = directory.write("current.json", &current_text);
        let output = compare(&baseline, &current, "json");
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(
            json(&output),
            serde_json::json!({"status": "passed", "violations": [], "errors": []})
        );
        assert_eq!(fs::read_to_string(&baseline).unwrap(), baseline_text);
        assert_eq!(fs::read_to_string(&current).unwrap(), current_text);
    }
}

#[test]
fn silent_changes_exit_one_in_every_format_without_rewriting_files() {
    let directory = Directory::new();
    let baseline_text = snapshot::<0>();
    let baseline = directory.write("baseline.json", &baseline_text);
    let mut changed = Snapshot::new();
    changed.add::<u16, 0>("test.value").unwrap();
    let current_text = changed.to_json().unwrap();
    let current = directory.write("current.json", &current_text);
    for format in ["text", "json", "github"] {
        let output = compare(&baseline, &current, format);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        if format == "json" {
            let value = json(&output);
            assert_eq!(value["status"], "failed");
            assert!(!value["violations"].as_array().unwrap().is_empty());
            assert_eq!(value["violations"][0]["id"], "test.value");
            assert!(value["errors"].as_array().unwrap().is_empty());
            assert_eq!(compare(&baseline, &current, format).stdout, output.stdout);
        } else if format == "github" {
            assert!(
                String::from_utf8(output.stdout)
                    .unwrap()
                    .contains("::error title=Schema version check::")
            );
        } else {
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains("test.value")
            );
        }
    }
    assert_eq!(fs::read_to_string(&baseline).unwrap(), baseline_text);
    assert_eq!(fs::read_to_string(&current).unwrap(), current_text);
}

#[test]
fn missing_and_malformed_snapshots_fail_closed() {
    let directory = Directory::new();
    let valid = directory.write("valid.json", &snapshot::<0>());
    let malformed = directory.write("malformed.json", "{not-json}");
    let missing = directory.0.join("missing.json");
    for (baseline, current) in [
        (&malformed, &valid),
        (&valid, &malformed),
        (&missing, &valid),
        (&valid, &missing),
    ] {
        let output = compare(baseline, current, "json");
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        let value = json(&output);
        assert_eq!(value["status"], "error");
        assert_eq!(value["errors"].as_array().unwrap().len(), 1);
    }
    assert!(!missing.exists());
}

#[test]
fn oversized_snapshots_return_a_bounded_input_error() {
    let directory = Directory::new();
    let valid = directory.write("valid.json", &snapshot::<0>());
    let oversized = directory.0.join("oversized.json");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(MAX_JSON_BYTES as u64 + 1)
        .unwrap();
    for (baseline, current) in [(&oversized, &valid), (&valid, &oversized)] {
        for format in ["json", "github"] {
            let output = compare(baseline, current, format);
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            if format == "json" {
                let value = json(&output);
                assert_eq!(value["status"], "error");
                assert!(
                    value["errors"][0]
                        .as_str()
                        .unwrap()
                        .contains("JSON exceeds")
                );
            } else {
                assert!(output.stderr.is_empty());
                let text = String::from_utf8(output.stdout).unwrap();
                assert_eq!(text.lines().count(), 1);
                assert!(text.starts_with("::error title=Schema version check::"));
                assert!(text.contains("JSON exceeds"));
            }
        }
    }
}

#[test]
fn invalid_arguments_exit_two() {
    for args in [
        vec![],
        vec!["export"],
        vec!["check"],
        vec!["check", "a", "b", "extra"],
        vec!["check", "a", "b", "--unknown"],
        vec!["check", "a", "b", "--format"],
        vec!["check", "a", "b", "--format", "xml"],
        vec!["check", "a", "b", "--format", "json", "--format", "text"],
    ] {
        let args: Vec<_> = args.iter().map(OsStr::new).collect();
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
    }
    let output = run(&[
        "check".as_ref(),
        "--unknown".as_ref(),
        "--format".as_ref(),
        "json".as_ref(),
    ]);
    assert_eq!(json(&output)["status"], "error");
}

#[test]
fn github_errors_cannot_inject_workflow_commands() {
    for args in [
        vec![
            "check",
            "--unknown%\r\n::notice::injected",
            "--format",
            "github",
        ],
        vec![
            "check",
            "missing%\r\n::notice::injected",
            "also-missing.json",
            "--format",
            "github",
        ],
    ] {
        let args: Vec<_> = args.iter().map(OsStr::new).collect();
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert_eq!(text.lines().count(), 1);
        assert!(text.starts_with("::error title=Schema version check::"));
        assert!(text.contains("%25%0D%0A::notice::injected"));
    }
}

#[test]
fn help_and_version_are_successful_without_input_files() {
    for argument in ["--help", "-h", "--version", "-V"] {
        let output = run(&[argument.as_ref()]);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("alkahest-schema")
        );
    }
}

#[test]
fn option_terminator_accepts_paths() {
    let directory = Directory::new();
    let path = directory.write("value.json", &snapshot::<0>());
    let output = run(&[
        "check".as_ref(),
        "--format".as_ref(),
        "json".as_ref(),
        "--".as_ref(),
        path.as_os_str(),
        path.as_os_str(),
    ]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(json(&output)["status"], "passed");
}
