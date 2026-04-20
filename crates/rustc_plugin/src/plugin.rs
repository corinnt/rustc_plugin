use std::{borrow::Cow, ops::ControlFlow, path::PathBuf, process::Command};

use cargo_metadata::camino::Utf8Path;
use rustc_middle::ty::TyCtxt;
use serde::{Serialize, de::DeserializeOwned};

/// Specification of a set of crates.
pub enum CrateFilter {
  /// A crate and all transitive dependencies. 
  CrateAndDeps,

  /// Every crate in the workspace and all transitive dependencies.
  AllCrates,

  /// Just crates in the workspace.
  OnlyWorkspace,

  /// Only the crate containing a specific file.
  CrateContainingFile(PathBuf),
}

/// Arguments from your plugin to the rustc_plugin framework.
pub struct RustcPluginArgs<Args> {
  /// Whatever CLI arguments you want to pass along.
  pub args: Args,

  /// Which crates you want to run the plugin on.
  pub filter: CrateFilter,
}

/// Interface between your plugin and the rustc_plugin framework.
pub trait RustcPlugin: Sized + Send {
  /// Command-line arguments passed by the user.
  type Args: Serialize + DeserializeOwned + Send;

  /// Returns the version of your plugin.
  ///
  /// A sensible default is your plugin's Cargo version:
  ///
  /// ```ignore
  /// env!("CARGO_PKG_VERSION").into()
  /// ```
  fn version(&self) -> Cow<'static, str>;

  /// Returns the name of your driver binary as it's installed in the filesystem.
  ///
  /// Should be just the filename, not the full path.
  fn driver_name(&self) -> Cow<'static, str>;

  /// Parses and returns the CLI arguments for the plugin.
  fn args(&self, target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args>;

  /// Optionally modify the `cargo` command that launches rustc.
  /// For example, you could pass a `--feature` flag here.
  fn modify_cargo(&self, _cargo: &mut Command, _args: &Self::Args) {}

  /// Optionally modify the compiler args passed to `rustc_public::run_with_tcx!`.
  /// For example, you could pass `-Zalways-encode-mir` or
  /// `-Zcrate-attr=register_tool(my_tool)` here.
  ///
  /// This is called in the driver process (once per crate), after plugin args
  /// have been deserialized but before the compiler is invoked.
  fn modify_compiler_args(&self, _args: &mut Vec<String>, _plugin_args: &Self::Args) {}

  /// Executes the plugin analysis with the compiler's TyCtxt.
  ///
  /// Called by the driver via `rustc_public::run_with_tcx!`. The framework
  /// handles compiler invocation; plugins only need to implement the analysis.
  ///
  /// Returns `ControlFlow::Break(())` to signal an error, or
  /// `ControlFlow::Continue(())` on success.
  fn run(
    self,
    plugin_args: Self::Args,
    tcx: TyCtxt<'_>,
  ) -> ControlFlow<()>;
}

/// The name of the environment variable shared between the CLI and the driver.
/// Must not conflict with any other env var used by Cargo.
pub const PLUGIN_ARGS: &str = "PLUGIN_ARGS";
