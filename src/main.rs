#![feature(globs)]

extern crate yaml;
use std::io::File;
mod specfile;

fn main() {
  let mut file = File::open(&Path::new("spi.yaml"));

  let doc = yaml::parse_io_utf8(&mut file).unwrap();
  let component: Result<specfile::Component, _> = specfile::FromYaml::from_yaml(&doc[0]);

  println!("{}", component);
}
