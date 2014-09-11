use yaml::constructor::*;

type YamlError = &'static str;

pub trait FromYaml {
  fn from_yaml(y: &YamlStandardData) -> Result<Self, YamlError>;
}

fn yaml_str<'s>(y: &'s YamlStandardData) -> Result<&'s str, YamlError> {
  match *y {
    YamlString(ref s) => Ok(s.as_slice()),
    _ => Err("Expected string"),
  }
}

fn yaml_pairs(y: &YamlStandardData) -> Result<&[(YamlStandardData, YamlStandardData)], YamlError> {
  match *y {
    YamlMapping(ref x) => Ok(x.as_slice()),
    _ => Err("Expected mapping")
  }
}

impl FromYaml for String {
  fn from_yaml(y: &YamlStandardData) -> Result<String, YamlError> {
    yaml_str(y).map(|s| s.to_string())
  }
}

#[deriving(Show)]
struct AssocList<K, V> {
  elems: Vec<(K, V)>
}

impl <K, V> AssocList<K, V> {
  fn new() -> AssocList<K, V> {
    AssocList{ elems: Vec::new() }
  }
  fn as_slice(&self) -> &[(K, V)] {
    self.elems.as_slice()
  }
  fn insert(&mut self, k: K, v: V) {
    self.elems.push((k, v));
  }
  fn iter<'s>(&'s self) -> ::std::slice::Items<'s, (K, V)> {
    self.elems.iter()
  }
}

impl <V> AssocList<String, V> {
  fn find_str<'s>(&'s self, key: &str) -> Option<&'s V> {
    for i in self.iter() {
      let &(ref k, ref v) = i;
      if k.as_slice() == key {
        return Some(v);
      }
    }
    None
  }
}

impl<K: FromYaml, V: FromYaml> FromYaml for AssocList<K, V> {
  fn from_yaml(y: &YamlStandardData) -> Result<AssocList<K, V>, YamlError> {
    let items = try!(yaml_pairs(y));
    let mut m = AssocList::new();
    for &(ref k, ref v) in items.iter() {
      m.insert(try!(FromYaml::from_yaml(k)), try!(FromYaml::from_yaml(v)))
    }
    Ok(m)
  }
}

#[test]
fn test_assoclist() {
  let mut l = AssocList::new();
  l.insert("foo".to_string(), 1i);
  l.insert("bar".to_string(), 2i);

  let mut iter = l.iter().map(|&(ref k, v)| (k.as_slice(), v));
  assert_eq!(iter.next(), Some(("foo", 1)));
  assert_eq!(iter.next(), Some(("bar", 2)));
  assert_eq!(iter.next(), None);

  assert_eq!(l.find_str("bar").map(|x| *x), Some(2));
  assert_eq!(l.find_str("foo").map(|x| *x), Some(1));
  assert_eq!(l.find_str("baz"), None);

}

#[deriving(Show)]
pub struct Component {
  name: String,
  backend: String,
  main: Action,
}

impl FromYaml for Component {
  fn from_yaml(y: &YamlStandardData) -> Result<Component, YamlError> {
    let mut name = Err("Missing name");
    let mut backend = Err("Missing backend");

    let items = try!(yaml_pairs(y));
    for &(ref k, ref v) in items.iter() {
      match try!(yaml_str(k)) {
        "component" => {
          name = FromYaml::from_yaml(v);
        }
        "backend" => {
          backend = FromYaml::from_yaml(v);
        }
        _ => ()
      }
    }

    Ok(Component {
      name: try!(name),
      backend: try!(backend),
      main: try!(FromYaml::from_yaml(y)),
    })
  }
}


#[deriving(Show)]
pub enum ActionEvent {
  NoEvent,
  EventCodeOn(String),
  EventNameTo(String),
}

#[deriving(Show)]
pub struct Action {
  args_in: AssocList<String, Argument>,
  args_out: AssocList<String, Argument>,
  actions: AssocList<String, Action>,
  begin: ActionEvent,
  end: ActionEvent,
}

impl FromYaml for Action {
  fn from_yaml(y: &YamlStandardData) -> Result<Action, YamlError> {
      let mut args_in = None;
      let mut args_out = None;
      let mut actions = None;
      let mut begin = NoEvent;
      let mut end = NoEvent;

      let items = try!(yaml_pairs(y));
      for &(ref k, ref v) in items.iter() {
        match try!(yaml_str(k)) {
          "args_in"  => args_in = Some(try!(FromYaml::from_yaml(v))),
          "args_out" => args_out = Some(try!(FromYaml::from_yaml(v))),
          "actions"  => actions = Some(try!(FromYaml::from_yaml(v))),
          "on_begin" => begin = EventCodeOn(try!(FromYaml::from_yaml(v))),
          "on_end"   => end = EventCodeOn(try!(FromYaml::from_yaml(v))),
          "to_begin" => begin = EventNameTo(try!(FromYaml::from_yaml(v))),
          "to_end"   => end = EventNameTo(try!(FromYaml::from_yaml(v))),
          _ => ()
        }
      }

      Ok(Action {
        args_in: args_in.unwrap_or_else(||AssocList::new()),
        args_out: args_out.unwrap_or_else(||AssocList::new()),
        actions: actions.unwrap_or_else(||AssocList::new()),
        begin: begin,
        end: end,
      })
  }
}

#[deriving(Show)]
pub enum Type {
  ByteType,
  IntType,
  FloatType,
  SymbolType,
  ComponentType,
  PtrType,
}


impl FromYaml for Type {
  fn from_yaml(y: &YamlStandardData) -> Result<Type, YamlError> {
    match try!(yaml_str(y)) {
      "byte" => Ok(ByteType),
      "int" => Ok(IntType),
      "float" => Ok(FloatType),
      "symbol" => Ok(SymbolType),
      "component" => Ok(ComponentType),
      "ptr" => Ok(ComponentType),
      _ => Err("Unknown type")
    }
  }
}

#[deriving(Show)]
pub struct Argument {
  ty: Type,
  actions: AssocList<String, Action>,
}

impl FromYaml for Argument {
  fn from_yaml(y: &YamlStandardData) -> Result<Argument, YamlError> {
    match *y {
      YamlString(..) => {
        Ok(Argument { ty: try!(FromYaml::from_yaml(y)), actions: AssocList::new() } )
      }
      YamlMapping(ref items) => {
        let mut actions = None;
        let ty = ComponentType;
        for &(ref k, ref v) in items.iter() {
          match try!(yaml_str(k)) {
            "actions" => actions = Some(try!(FromYaml::from_yaml(v))),
            _ => ()
          }
        }
        Ok( Argument{ ty: ty, actions: actions.unwrap_or_else(|| AssocList::new() ) } )
      }
      _ => Err("Invalid argument type")
    }
  }
}
