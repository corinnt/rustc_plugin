//! A Rustc plugin that prints out all local variable types in each function in a crate.
//! Unlike the print-all-items example, this analysis uses the rustc_public stable-MIR
//! API exclusively — no HIR traversal.

#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;
extern crate rustc_session;

use std::{borrow::Cow, env, ops::ControlFlow, process::Command};

use clap::Parser;
use rustc_middle::ty::TyCtxt;
use rustc_plugin::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use rustc_public::{CrateDef, ItemKind, all_local_items, mir::mono::Instance};
use serde::{Deserialize, Serialize};

pub struct PrintTypesPlugin;

#[derive(Parser, Serialize, Deserialize, Clone)]
pub struct PrintTypesPluginArgs {
  #[clap(last = true)]
  cargo_args: Vec<String>,
}

impl RustcPlugin for PrintTypesPlugin {
  type Args = PrintTypesPluginArgs;

  fn version(&self) -> Cow<'static, str> {
    env!("CARGO_PKG_VERSION").into()
  }

  fn driver_name(&self) -> Cow<'static, str> {
    "print-types-driver".into()
  }

  fn args(&self, _target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args> {
    let args = PrintTypesPluginArgs::parse_from(env::args().skip(1));
    RustcPluginArgs { args, filter: CrateFilter::AllCrates }
  }

  fn modify_cargo(&self, cargo: &mut Command, args: &Self::Args) {
    cargo.args(&args.cargo_args);
  }

  fn run(self, _plugin_args: Self::Args, _tcx: TyCtxt<'_>) -> ControlFlow<()> {
    for item in &all_local_items() {
      if !matches!(item.kind(), ItemKind::Fn) {
        continue;
      }
      let Ok(instance) = Instance::try_from(*item) else {
        continue;
      };
      let Some(body) = instance.body() else {
        continue;
      };
      println!("fn {}", item.name());
      for local in body.locals() {
        println!("  type: {}", local.ty);
      }
    }
    ControlFlow::Continue(())
  }
}
