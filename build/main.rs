//! See the readme.md in this directory for an overview of this build script.
use anyhow::Context;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
enum ParseError {
    Dtype(String),
    Otype(String),
    CsvFile(Option<csv::Position>),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ParseError::Dtype(s) => &format!("error parsing data type: {s}"),
            ParseError::Otype(s) => &format!("error parsing object type: {s}"),
            ParseError::CsvFile(Some(pos)) => &format!(
                "error parsing CSV record {} (line {}, byte {})",
                pos.record(),
                pos.line(),
                pos.byte()
            ),
            ParseError::CsvFile(None) => "error parsing CSV",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for ParseError {}

#[derive(Hash, Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
enum DataType {
    #[serde(rename = "dbl")]
    Double,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "chr")]
    Char,
    #[serde(rename = "str")]
    Str,
    #[serde(rename = "custom")]
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
                return Err(ParseError::Dtype(
                    "expected `custom`, `int`, `chr`, `str` or `dbl`".to_string(),
                ));
            }
        };
        Ok(dt)
    }
}

impl DataType {
    fn ident_fragment(self) -> Option<&'static str> {
        use DataType::*;
        match self {
            Double => Some("Double"),
            Int => Some("Int"),
            Char => Some("Char"),
            Str => Some("Str"),
            Custom => None,
        }
    }

    fn doc_description(self) -> Option<&'static str> {
        use DataType::*;
        match self {
            Double => Some("double (`f64`)"),
            Int => Some("integer (`i32`)"),
            Char => Some("`char`"),
            Str => Some("string (`String`)"),
            Custom => None,
        }
    }
}

#[derive(Hash, Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ObjType {
    Model,
    Var,
    Constr,
    #[serde(rename = "gconstr")]
    GenConstr,
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
            "gconstr" => ObjType::GenConstr,
            "qconstr" => ObjType::QConstr,
            "sos" => ObjType::SOS,
            _ => {
                return Err(ParseError::Otype(
                    "expected `model`, `var`, `constr`, `gconstr`, `qconstr` or `sos`".to_string(),
                ));
            }
        };
        Ok(dt)
    }
}

impl ObjType {
    fn obj_ident(self) -> Ident {
        format_ident!("{}", format!("{:?}", self))
    }

    fn needs_objattr_impl(self) -> bool {
        !matches!(self, ObjType::Model)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParameterMeta {
    url: String,
    name: String,
    dtype: DataType,
    default: String,
    min: Option<String>,
    max: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AttributeMeta {
    url: String,
    name: String,
    modifiable: bool,
    otype: ObjType,
    dtype: DataType,
}

fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let path = path.as_ref();
    let reader = std::io::BufReader::new(
        std::fs::File::open(path).with_context(|| format!("unable to read {path:?}"))?,
    );
    let val = serde_json::from_reader(reader)?;
    Ok(val)
}

fn get_docstring_body(name: &str, suffix: &str) -> anyhow::Result<String> {
    let path = format!("build/docstrings/body/{name}_{suffix}.md");
    let body =
        std::fs::read_to_string(&path).with_context(|| format!("unable to read {path:?}"))?;
    Ok(body)
}

fn parse_csv<T, P>(filename: &impl AsRef<Path>, row_parser: P) -> anyhow::Result<Vec<T>>
where
    P: Fn(&csv::StringRecord) -> anyhow::Result<T>,
{
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_path(filename)?;
    let mut values = Vec::new();
    for result in rdr.records() {
        let row = result?;
        let val = row_parser(&row).with_context(|| ParseError::CsvFile(row.position().cloned()))?;
        values.push(val);
    }
    Ok(values)
}

pub fn str_to_ident(s: &str) -> Ident {
    Ident::new(s, proc_macro2::Span::call_site())
}

pub fn docstring_filepath(name: &str) -> String {
    let path = format!("build/docstrings/final/{name}.md");
    eprintln!("{path}");
    Path::new(&path)
        .canonicalize()
        .unwrap_or_else(|_| {
            Path::new("build/docstrings/missing.md")
                .canonicalize()
                .unwrap()
        })
        .into_os_string()
        .into_string()
        .unwrap()
}

type ParameterEnums = HashMap<Ident, (DataType, Vec<String>)>;
type AttributeEnums = HashMap<Ident, (ObjType, DataType, Vec<String>)>;

mod param {
    use super::*;
    pub(crate) fn parse_csv_row(row: &csv::StringRecord) -> anyhow::Result<(DataType, String)> {
        if row.len() != 2 {
            anyhow::bail!("row should have 2 fields");
        }
        let dtype: DataType = row[1].parse()?;
        let name = row[0].to_string();
        Ok((dtype, name))
    }

    fn get_metadata(name: &str) -> anyhow::Result<ParameterMeta> {
        load_json(format!("build/docstrings/metadata/{name}_param.json"))
    }

    fn build_docstring(name: &str) -> anyhow::Result<String> {
        let body = get_docstring_body(name, "param");
        if let Ok(body) = body {
            let meta = get_metadata(name)?;
            let mut docstring = String::new();

            if let Some(val) = meta.dtype.doc_description() {
                writeln!(docstring, "- __Type:__ {val}")?;
            }
            writeln!(docstring, "- __Default:__ {}", &meta.default)?;
            if let Some(val) = &meta.min {
                writeln!(docstring, "- __Minimum:__ {val}")?;
            }
            if let Some(val) = &meta.max {
                writeln!(docstring, "- __Maximum:__ {val}")?;
            }

            docstring.push_str("\n\n");
            docstring.push_str(&body);
            docstring.push_str("\n\n");
            writeln!(docstring, "[Reference manual]({}).", &meta.url)?;
            Ok(docstring)
        } else {
            // body?;
            let fallback = std::fs::read_to_string("build/docstrings/missing.md")?;
            Ok(fallback)
        }
    }

    fn gen_variant(name: &str) -> TokenStream {
        let ident = str_to_ident(name);
        let docstring = build_docstring(name).unwrap();
        quote! {
          #[doc=#docstring]
          #ident
        }
    }

    fn gen_type(ts: &mut TokenStream, ident: &Ident, members: &[String]) -> anyhow::Result<()> {
        let members = members.iter().map(|s| gen_variant(&*s));

        let decl = quote! {
          #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, FromCStr, AsCStr)]
          pub enum #ident {
            #(
              #members
            ),*
          }
        };
        ts.extend(decl);
        Ok(())
    }

    pub(super) fn generate_src(
        path: impl AsRef<Path>,
        enums: &ParameterEnums,
    ) -> anyhow::Result<()> {
        let mut ts = quote! {
          use cstr_enum::*;
        };

        for (ident, (_, members)) in enums {
            gen_type(&mut ts, ident, members)?;
        }

        let exports: Vec<_> = enums.keys().collect();
        ts.extend(quote! {
          pub(super) mod enum_exports {
            #(
              pub use super::#exports;
            )*
          }

          pub mod variant_exports {
            #(
              #[doc(inline)]
              pub use super::#exports::*;
            )*
          }
        });

        std::fs::write(path, ts.to_string())?;
        Ok(())
    }

    pub(crate) fn group_into_enums(
        attrs: impl IntoIterator<Item = (DataType, String)>,
    ) -> ParameterEnums {
        let mut map = ParameterEnums::new();
        for (dt, name) in attrs {
            let ident = dt
                .ident_fragment()
                .map(|dt| format_ident!("{}Param", dt))
                .expect("not implemented");

            match map.entry(ident) {
                Entry::Occupied(mut e) => {
                    let (_, members) = e.get_mut();
                    members.push(name);
                }
                Entry::Vacant(e) => {
                    e.insert((dt, vec![name]));
                }
            }
        }
        map
    }
}

mod attrs {
    use super::*;
    pub(crate) fn parse_csv_row(
        row: &csv::StringRecord,
    ) -> anyhow::Result<(ObjType, DataType, String)> {
        if row.len() != 3 {
            anyhow::bail!("row should have 3 fields");
        }
        let obj: ObjType = row[2].parse()?;
        let dtype: DataType = row[1].parse()?;
        let name = row[0].to_string();
        Ok((obj, dtype, name))
    }

    fn get_metadata(name: &str) -> anyhow::Result<AttributeMeta> {
        load_json(format!("build/docstrings/metadata/{name}_attr.json"))
    }

    fn build_docstring(name: &str) -> anyhow::Result<String> {
        let body = get_docstring_body(name, "attr");
        if let Ok(body) = body {
            let meta = get_metadata(name)?;
            let mut docstring = String::new();

            writeln!(
                docstring,
                "- __Modifiable:__ {}",
                if meta.modifiable { "Yes" } else { "No" }
            )?;
            if let Some(val) = meta.dtype.doc_description() {
                writeln!(docstring, "- __Type:__ {val}")?;
            }

            docstring.push_str("\n\n");
            docstring.push_str(&body);
            docstring.push_str("\n\n");
            writeln!(docstring, "[Reference manual]({}).", &meta.url)?;
            Ok(docstring)
        } else {
            body?;
            let fallback = std::fs::read_to_string("build/docstrings/missing.md")?;
            Ok(fallback)
        }
    }

    fn gen_variant(name: &str) -> TokenStream {
        let ident = str_to_ident(name);
        let docstring = build_docstring(name).unwrap();
        quote! {
          #[doc=#docstring]
          #ident
        }
    }

    fn add_dtype_marker_impl(ts: &mut TokenStream, target: &Ident, d: DataType) {
        let ident = format_ident!("{}Attr", d.ident_fragment().unwrap());
        ts.extend(quote! {
          impl #ident for #target {}
        });
    }

    fn add_otype_marker_impl(ts: &mut TokenStream, target: &Ident, o: ObjType) {
        let ident = o.obj_ident();
        ts.extend(quote! {
          impl ObjAttr for #target {
            type Obj = #ident;
          }
        });
    }

    fn gen_type(
        ts: &mut TokenStream,
        ident: &Ident,
        d: DataType,
        o: ObjType,
        members: &[String],
    ) -> anyhow::Result<()> {
        let variants = members.iter().map(|s| gen_variant(&*s));
        ts.extend(quote! {
          #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, FromCStr, AsCStr)]
          pub enum #ident {
            #(
              #variants
            ),*
          }
        });

        if !matches!(d, DataType::Custom) {
            add_dtype_marker_impl(ts, ident, d);
        } else {
            assert_eq!(members.len(), 1);
        }

        if o.needs_objattr_impl() {
            add_otype_marker_impl(ts, ident, o);
        }

        Ok(())
    }

    pub(super) fn generate_src(
        path: impl AsRef<Path>,
        enums: &AttributeEnums,
    ) -> anyhow::Result<()> {
        let mut ts = quote! {
          use cstr_enum::*;
          use super::{IntAttr, CharAttr, StrAttr, DoubleAttr, ObjAttr, Var, Constr, GenConstr, QConstr, SOS};
        };

        for (ident, (o, d, members)) in enums {
            gen_type(&mut ts, ident, *d, *o, members)?;
        }

        let exports: Vec<_> = enums.keys().collect();
        ts.extend(quote! {
          pub(super) mod enum_exports {
            #(
              pub use super::{#exports};
            )*
          }

          pub mod variant_exports {
            #(
              pub use super::#exports::*;
            )*
          }
        });

        std::fs::write(path, ts.to_string())?;
        Ok(())
    }

    pub(crate) fn group_into_enums(
        attrs: impl IntoIterator<Item = (ObjType, DataType, String)>,
    ) -> AttributeEnums {
        let mut map = AttributeEnums::new();
        for (ot, dt, name) in attrs {
            let ident = dt.ident_fragment().map_or_else(
                || format_ident!("{}{}Attr", ot.obj_ident(), &name),
                |dt| format_ident!("{}{}Attr", ot.obj_ident(), dt),
            );

            match map.entry(ident) {
                Entry::Occupied(mut e) => {
                    let (_, _, members) = e.get_mut();
                    members.push(name);
                }
                Entry::Vacant(e) => {
                    e.insert((ot, dt, vec![name]));
                }
            }
        }
        map
    }
}

#[allow(dead_code)]
fn debug_src_code(src: &TokenStream) -> anyhow::Result<()> {
    std::fs::write("build/debug.rs", src.to_string())?;
    std::process::Command::new("rustfmt")
        .arg("build/debug.rs")
        .output()?;
    Ok(())
}

fn get_data_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    p.push("build");
    p.push(name);
    p
}

fn get_output_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();
    p.push(name);
    p
}

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build/params.csv");
    println!("cargo:rerun-if-changed=build/main.rs");
    println!("cargo:rerun-if-changed=build/attrs.csv");
    println!("cargo:rerun-if-changed=build/docstrings");

    let attr_data = get_data_path("attrs.csv");
    let param_data = get_data_path("params.csv");
    let param_list =
        parse_csv(&param_data, param::parse_csv_row).context("failed to read parameter CSV")?;
    let enums = param::group_into_enums(param_list);
    param::generate_src(get_output_path("param_enums.rs"), &enums)?;

    let attr_list =
        parse_csv(&attr_data, attrs::parse_csv_row).context("failed to read attribute CSV")?;
    let enums = attrs::group_into_enums(attr_list);
    attrs::generate_src(get_output_path("attr_enums.rs"), &enums)?;

    Ok(())
}
