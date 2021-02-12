use std::path::Path;
use std::str::FromStr;
use std::io::{Write, self};
use std::fs;
use std::fmt;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::hash::Hash;
use anyhow::Context;
use codegen;
use csv;
use codegen::Enum;

#[derive(Debug, Clone, Eq, PartialEq)]
enum ParseError {
  Dtype(String),
  Otype(String),
  CsvFile(Option<csv::Position>),
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ParseError::Dtype(s) => f.write_fmt(format_args!("error parsing data type: {}", s)),
      ParseError::Otype(s) => f.write_fmt(format_args!("error parsing object type: {}", s)),
      ParseError::CsvFile(Some(pos)) => f.write_fmt(format_args!("error parsing CSV record {} (line {}, byte {})", pos.record(), pos.line(), pos.byte())),
      ParseError::CsvFile(None) => f.write_fmt(format_args!("error parsing CSV")),
    }
  }
}

impl std::error::Error for ParseError {}

#[derive(Hash, Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
enum DataType {
  Double,
  Int,
  Char,
  Str,
  Custom,
}

impl FromStr for DataType {
  type Err = ParseError;
  fn from_str(s: &str) -> Result<DataType, Self::Err> {
    let dt = match s {
      "custom" => DataType::Custom,
      "int" => DataType::Int,
      "chr" => DataType::Char,
      "str" => DataType::Str,
      "dbl" => DataType::Double,
      _ => {
        return Err(ParseError::Dtype("expected `custom`, `int`, `chr`, `str` or `dbl`".to_string()))
      }
    };
    Ok(dt)
  }
}

#[derive(Hash, Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
enum ObjType {
  Model,
  Var,
  Constr,
  QConstr,
  SOS,
}


impl FromStr for ObjType {
  type Err = ParseError;
  fn from_str(s: &str) -> Result<ObjType, Self::Err> {
    let dt = match s {
      "model" => ObjType::Model,
      "var" => ObjType::Var,
      "constr" => ObjType::Constr,
      "qconstr" => ObjType::QConstr,
      "sos" => ObjType::SOS,
      _ => {
        return Err(ParseError::Otype("expected `model`, `var`, `constr`, `qconstr` or `sos`".to_string()))
      }
    };
    Ok(dt)
  }
}

fn parse_attr_row(row: &csv::StringRecord) -> anyhow::Result<((ObjType, DataType), String)> {
  if row.len() != 3 {
    anyhow::bail!("row should have 3 fields");
  }
  let obj: ObjType = row[2].parse()?;
  let dtype: DataType = row[1].parse()?;
  let name = row[0].to_string();
  Ok(((obj, dtype), name))
}

fn parse_param_row(row: &csv::StringRecord) -> anyhow::Result<(DataType, String)> {
  if row.len() != 2 {
    anyhow::bail!("row should have 2 fields");
  }
  let dtype: DataType = row[1].parse()?;
  let name = row[0].to_string();
  Ok((dtype, name))
}

fn parse_csv<K, V, P>(filename: &impl AsRef<Path>, row_parser: P) -> anyhow::Result<Vec<(K, Vec<V>)>>
  where
    K: Hash + Eq + Ord,
    V: Ord,
    P: Fn(&csv::StringRecord) -> anyhow::Result<(K,V)>,
{
  // Build the CSV reader and iterate over each record.
  let mut rdr = csv::Reader::from_path(filename)?;
  let mut grouped : HashMap<_, Vec<V>> = HashMap::default();

  for result in rdr.records() {
    let row = result?;
    let (k,v) = row_parser(&row)
      .with_context(|| ParseError::CsvFile(row.position().cloned()))?;
    grouped.entry(k).or_default().push(v);
  }

  let mut grouped : Vec<_> = grouped.into_iter().collect();
  grouped.sort();

  Ok(grouped)
}

fn add_enum_derives(e: &mut Enum) {
  for t in &["Debug", "Copy", "Clone", "Eq", "PartialEq", "Hash", "FromCStr", "AsCStr"] {
    e.derive(t);
  }
}

fn make_custom_attr_enum(o: ObjType, member: &String) -> (String, codegen::Enum) {
  let name = format!("{:?}{}Attr", o, member);
  let mut e = codegen::Enum::new(&name);
  e.vis("pub");
  e.doc(&format!("Gurobi `{}` attribute for [`{:?}`](crate::{:?}) objects.",&member, o, o));
  add_enum_derives(&mut e);
  e.new_variant(member);
  (name, e)
}

fn make_attr_enum(o: ObjType, d: DataType, members: &Vec<String>) -> (String, codegen::Enum) {
  let name = format!("{:?}{:?}Attr", o, d);
  let mut e = codegen::Enum::new(&name);
  e.vis("pub");
  e.doc(&format!("{} Gurobi attributes for [`{:?}`](crate::{:?}) objects.",
        match d {
          DataType::Char => "Char",
          DataType::Int => "Integer",
          DataType::Double => "Float",
          DataType::Str => "String",
          _ => unreachable!(),
        },
        o, o
  ));
  add_enum_derives(&mut e);
  for m in members {
    e.new_variant(m);
  }
  (name, e)
}

fn make_custom_param_enum(paramname: &String) -> (String, codegen::Enum) {
  let name = format!("{}Param", paramname);
  let mut e = codegen::Enum::new(&name);
  e.vis("pub");
  e.doc(&format!("Gurobi parameter `{}`",&paramname));
  add_enum_derives(&mut e);
  e.new_variant(paramname);
  (name, e)
}

fn make_param_enum(d: DataType, members: &Vec<String>) -> (String, codegen::Enum) {
  let name = format!("{:?}Param", d);
  let mut e = codegen::Enum::new(&name);
  e.vis("pub");
  e.doc(&format!("{} Gurobi parameters",
        match d {
          DataType::Char => "Char",
          DataType::Int => "Integer",
          DataType::Double => "Float",
          DataType::Str => "String",
          _ => unreachable!(),
        },
  ));
  add_enum_derives(&mut e);
  for m in members {
    e.new_variant(m);
  }
  (name, e)
}

fn try_rustfmt_file(filename: &impl AsRef<OsStr>) {
  #![allow(unused_must_use)]
  std::process::Command::new("rustfmt")
    .arg(&filename)
    .output().unwrap();
}



fn add_shared_imports(scope: &mut codegen::Scope) {
  scope.import("cstr_enum", "*");
}



fn generate_param_src_file(filename: &impl AsRef<Path>, grouped_params: Vec<(DataType, Vec<String>)>) -> anyhow::Result<()> {
  let mut scope = codegen::Scope::new();
  add_shared_imports(&mut scope);

  let mut enums = Vec::new();

  for (d, paramnames) in grouped_params {
    if d == DataType::Custom {
      for paramname in paramnames {
        let (name, enm) = make_custom_param_enum(&paramname);
        enums.push(name);
        scope.push_enum(enm);
      }
    } else {
      let (name, enm) = make_param_enum(d, &paramnames);
      enums.push(name);
      scope.push_enum(enm);
    }
  }

  let enum_exports = scope.new_module("enum_exports").vis("pub(super)").scope();
  for ename in &enums {
    enum_exports.import("super", ename).vis("pub");
  }
  let variant_exports = scope.new_module("variant_exports").vis("pub").scope();
  for ename in &enums {
    variant_exports.import(&format!("super::{}", ename), "*").vis("pub");
  }

  let mut output = io::BufWriter::new(
    fs::OpenOptions::new().write(true).truncate(true).create(true).open(filename)?
  );

  let doc = "//! This file is automatically generated - do not edit it.  \
    To add new Gurobi paramteters, edit the params.csv file instead.\n\n";

  writeln!(&mut output, "#![allow(missing_docs)]")?;
  write!(&mut output, "{}", doc)?;
  write!(&mut output, "{}", scope.to_string())?;
  Ok(())
}


fn generate_attr_src_file(filename: &impl AsRef<Path>, grouped_attrs: Vec<((ObjType, DataType), Vec<String>)>) -> anyhow::Result<()> {
  let mut scope = codegen::Scope::new();
  add_shared_imports(&mut scope);

  let typetrait : HashMap<DataType, String> = {
    let mut m = HashMap::default();

    let mut add_trait = |d, name: &str| {
      scope.import("super", name);
      m.insert(d, name.to_string());
    };

    add_trait(DataType::Int, "IntAttr");
    add_trait(DataType::Char, "CharAttr");
    add_trait(DataType::Str, "StrAttr");
    add_trait(DataType::Double, "DoubleAttr");
    m
  };

  let objtrait= "ObjAttr";
  scope.import("super", objtrait);


  let mut enums = Vec::new();

  for ((o, d), attrnames) in grouped_attrs {
    if d == DataType::Custom {
      for aname in attrnames {
        let (name, enm) = make_custom_attr_enum(o, &aname);
        scope.push_enum(enm);
        enums.push(name);
      }
    } else {
      let (name, enm) = make_attr_enum(o, d, &attrnames);
      enums.push(name);
      let enum_ty = enm.ty().clone();

      // Implement the datatype marker trait
      let mut impl_tytrait = codegen::Impl::new(&enum_ty);
      impl_tytrait.impl_trait(&typetrait[&d]);
      scope.push_enum(enm);
      scope.push_impl(impl_tytrait);

      if o != ObjType::Model {
        // Implement the model object marker trait
        let mut impl_otrait = codegen::Impl::new(&enum_ty);
        impl_otrait.impl_trait(objtrait);
        let obj_ty = format!("{:?}", o);
        scope.import("super", &obj_ty);
        impl_otrait.associate_type("Obj", &obj_ty);
        scope.push_impl(impl_otrait);
      }
    }
  }


  let enum_exports = scope.new_module("enum_exports").vis("pub(super)").scope();
  for ename in &enums {
    enum_exports.import("super", ename).vis("pub");
  }
  let variant_exports = scope.new_module("variant_exports").vis("pub").scope();
  for ename in &enums {
    variant_exports.import(&format!("super::{}", ename), "*").vis("pub");
  }

  let mut output = io::BufWriter::new(
    fs::OpenOptions::new().write(true).truncate(true).create(true).open(filename)?
  );


  let doc = "//! This file is automatically generated - do not edit it.  \
    To add new Gurobi attributes, edit the attrs.csv file instead.\n\n";

  writeln!(&mut output, "#![allow(missing_docs)]")?;
  write!(&mut output, "{}", doc)?;
  write!(&mut output, "{}", scope.to_string())?;
  Ok(())
}

const ATTR_SRC_FILE: &'static str = "src/attribute/attr_enums.rs";
const PARAM_SRC_FILE: &'static str = "src/parameter/param_enums.rs";
const ATTR_DATA : &'static str = "build/attrs.csv";
const PARAM_DATA : &'static str = "build/params.csv";

fn main() -> anyhow::Result<()> {
  println!("cargo:rerun-if-changed={}", &ATTR_DATA);
  println!("cargo:rerun-if-changed={}", &PARAM_DATA);
  println!("cargo:rerun-if-changed={}", "build/main.rs");
  let attr_groups = parse_csv(&ATTR_DATA, parse_attr_row)?;
  generate_attr_src_file(&ATTR_SRC_FILE, attr_groups)?;

  let param_groups = parse_csv(&PARAM_DATA, parse_param_row)?;
  generate_param_src_file(&PARAM_SRC_FILE, param_groups)?;

  try_rustfmt_file(&ATTR_SRC_FILE);
  try_rustfmt_file(&PARAM_SRC_FILE);
  Ok(())
}
