//! A Rustc plugin that prints out the name of all items in a crate.

#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;
extern crate rustc_session;

use std::{borrow::Cow, env, ops::ControlFlow, process::Command};

use clap::Parser;
use rustc_hir::{
  Item,
  intravisit::{self, Visitor},
};
use rustc_middle::ty::TyCtxt;
use rustc_plugin::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use serde::{Deserialize, Serialize};

// This struct is the plugin provided to the rustc_plugin framework,
// and it must be exported for use by the CLI/driver binaries.
pub struct PrintAllItemsPlugin;

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize, Clone)]
pub struct PrintAllItemsPluginArgs {
  #[arg(short, long)]
  allcaps: bool,

  #[clap(last = true)]
  cargo_args: Vec<String>,
}

impl RustcPlugin for PrintAllItemsPlugin {
  type Args = PrintAllItemsPluginArgs;

  fn version(&self) -> Cow<'static, str> {
    env!("CARGO_PKG_VERSION").into()
  }

  fn driver_name(&self) -> Cow<'static, str> {
    "print-all-items-driver".into()
  }

  // In the CLI, we ask Clap to parse arguments and also specify a CrateFilter.
  // If one of the CLI arguments was a specific file to analyze, then you
  // could provide a different filter.
  fn args(&self, _target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args> {
    let args = PrintAllItemsPluginArgs::parse_from(env::args().skip(1));
    let filter = CrateFilter::AllCrates;
    RustcPluginArgs { args, filter }
  }

  // Pass Cargo arguments (like --feature) from the top-level CLI to Cargo.
  fn modify_cargo(&self, cargo: &mut Command, args: &Self::Args) {
    cargo.args(&args.cargo_args);
  }

  // The driver invokes this via rustc_public::run_with_tcx!, providing
  // the TyCtxt directly. We just perform our analysis here.
  fn run(
    self,
    plugin_args: Self::Args,
    tcx: TyCtxt<'_>,
  ) -> ControlFlow<()> {
    print_all_items(tcx, plugin_args);
    ControlFlow::Continue(())
  }
}

// The core of our analysis. Right now it just prints out a description of each item.
// I recommend reading the Rustc Development Guide to better understand which compiler APIs
// are relevant to whatever task you have.
fn print_all_items(tcx: TyCtxt, args: PrintAllItemsPluginArgs) {
  tcx.hir_visit_all_item_likes_in_crate(&mut PrintVisitor { args, tcx });
}

struct PrintVisitor<'tcx> {
  args: PrintAllItemsPluginArgs,
  tcx: TyCtxt<'tcx>,
}

impl<'tcx> Visitor<'tcx> for PrintVisitor<'tcx> {
  fn visit_item(&mut self, item: &'tcx Item<'tcx>) -> Self::Result {
    let mut msg = match item.kind.ident() {
      Some(ident) => format!(
        "There is an item \"{}\" of type \"{}\"",
        ident,
        self.tcx.def_descr(item.owner_id.to_def_id())
      ),
      None => format!(
        "There is an item of type \"{}\"",
        self.tcx.def_descr(item.owner_id.to_def_id())
      ),
    };
    if self.args.allcaps {
      msg = msg.to_uppercase();
    }
    println!("{msg}");

    intravisit::walk_item(self, item)
  }
}
