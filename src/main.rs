#![feature(globs)]

extern crate yaml;
use std::io::File;
use std::os;
mod specfile;

fn main() {
  let args = os::args();
  let mut file = File::open(&Path::new(args[1].as_slice()));

  let doc = yaml::parse_io_utf8(&mut file).unwrap();
  let component: Result<specfile::Component, _> = specfile::FromYaml::from_yaml(&doc[0]);

  println!("{}", component);
}
