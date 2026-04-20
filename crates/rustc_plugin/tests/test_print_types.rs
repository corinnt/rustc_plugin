use std::{env, fs, path::Path, process::Command, sync::Once};

use anyhow::{Context, Result, ensure};

static SETUP: Once = Once::new();

fn run(dir: &str, f: impl FnOnce(&mut Command)) -> Result<String> {
  let root = env::temp_dir().join("rustc_plugin_print_types");

  let heredir = Path::new(".").canonicalize()?;

  SETUP.call_once(|| {
    let mut cmd = Command::new("cargo");
    cmd.args([
      "install",
      "--path",
      "examples/print-types",
      "--debug",
      "--locked",
      "--root",
    ]);
    cmd.arg(&root);
    cmd.current_dir(&heredir);
    let status = cmd.status().unwrap();
    if !status.success() {
      panic!("installing print-types example failed")
    }
  });

  let mut cmd = Command::new("cargo");
  cmd.arg("print-types");

  let path = format!(
    "{}:{}",
    root.join("bin").display(),
    env::var("PATH").unwrap_or_else(|_| "".into())
  );
  cmd.env("PATH", path);

  let ws = heredir.join("tests").join(dir);
  cmd.current_dir(&ws);

  f(&mut cmd);

  let _ = fs::remove_dir_all(ws.join("target"));

  let output = cmd.output().context("Process failed")?;
  ensure!(
    output.status.success(),
    "Process exited with non-zero exit code. Stderr:\n{}",
    String::from_utf8(output.stderr)?
  );

  Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn basic_types() -> Result<()> {
  let output = run("workspaces/basic", |_cmd| {})?;
  assert!(output.contains("fn add"), "output:\n{output}");
  assert!(output.contains("usize"), "output:\n{output}");
  Ok(())
}
