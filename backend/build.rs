extern crate embed_resource;
use std::env;


fn main() {
  if env::var("CARGO_CFG_TARGET_OS").unwrap().eq("windows") {
    embed_resource::compile("manifest.rc");
  }
}
