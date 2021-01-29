use quote::quote;
use proc_macro::{TokenStream};
use syn;
use syn::{Result, Error};
use std::str::FromStr;
use proc_macro2::{TokenStream as TokenStream2};

fn parse_cmp_expr(cmpexpr: syn::ExprBinary) -> Result<(Box<syn::Expr>, TokenStream2, Box<syn::Expr>)> {
    use syn::BinOp::*;
    let sense = match cmpexpr.op {
        Eq(..) => TokenStream2::from_str("::gurobi::Equal").unwrap(),
        Le(..) => TokenStream2::from_str("::gurobi::Less").unwrap(),
        Ge(..) => TokenStream2::from_str("::gurobi::Greater").unwrap(),
        Lt(..) | Gt(..) | Ne(..) => { return Err(Error::new_spanned(cmpexpr.op, "expected >=, <= or ==")) }
        _ => { return Err(Error::new_spanned(cmpexpr, "expression should be a ==, >= or <= comparison")) }
    };
    Ok((cmpexpr.left, sense, cmpexpr.right))
}

/// A proc-macro for creating [`constr::ConstrExpr`] objects.
///
/// The syntax is `c!( lhs CMP rhs )` where `CMP` is one of: `==`, `>=` or `<=`.
#[proc_macro]
pub fn c(expr: TokenStream) -> TokenStream {
    let cmpexp = syn::parse_macro_input!(expr as syn::ExprBinary);

    parse_cmp_expr(cmpexp)
      .map_or_else(
        |err| err.to_compile_error(),
        |(lhs, sense, rhs)| {
          let output = quote! {
              ::gurobi::constr::ConstrExpr{
                  lhs: ::gurobi::Expr::from(#lhs),
                  sense: #sense,
                  rhs: ::gurobi::Expr::from(#rhs),
              }
          };
          output.into()
      })
      .into()
}
