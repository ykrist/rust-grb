#![allow(unused_imports)] // TODO remove
#![allow(dead_code)] // TODO remove
use quote::{ToTokens, quote, TokenStreamExt};
use proc_macro::{TokenStream};
use syn;
use syn::{Token, Result, Error, ExprBinary, Expr};
use proc_macro2::{TokenStream as TokenStream2, TokenTree, Ident, Span};
use syn::parse::{ParseStream, Parse, Parser, discouraged::Speculative};
use syn::token::Token;
use crate::ConstrExpr::{Inequality, Range};
use syn::group::Group;
use std::str::FromStr;

struct InequalityConstr {
  lhs : Box<Expr>,
  sense: TokenStream2,
  rhs : Box<Expr>,
}

impl Parse for InequalityConstr {
  fn parse(input: ParseStream) -> Result<Self> {
    use syn::BinOp::*;

    let cmpexpr: syn::ExprBinary = input.parse()?;
    let sense = match cmpexpr.op {
      Eq(..) => quote! { gurobi::Equal },
      Le(..) => quote! { gurobi::Less },
      Ge(..) => quote! { gurobi::Greater },
      Lt(..) | Gt(..) | Ne(..) => { return Err(Error::new_spanned(cmpexpr.op, "expected >=, <= or ==")); }
      _ => { return Err(Error::new_spanned(cmpexpr, "expression should be a ==, >= or <= comparison")); }
    };

    Ok(InequalityConstr {lhs: cmpexpr.left, sense, rhs:cmpexpr.right})
    }
}

impl ToTokens for InequalityConstr {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    // let Self{ ref lhs, ref rhs, ref sense } = self;
    let lhs = &self.lhs;
    let rhs = &self.rhs;
    let sense = &self.sense;
    let ts = quote! {
      gurobi::constr::IneqExpr{
        lhs: gurobi::Expr::from(#lhs),
        sense: #sense,
        rhs: gurobi::Expr::from(#rhs),
      }
    };
    ts.to_tokens(tokens);
  }
}

struct GrbRangeExpr {
  lb: Option<Box<syn::Expr>>,
  ub: Option<Box<syn::Expr>>,
}

impl Parse for GrbRangeExpr {
  fn parse(input: ParseStream) -> Result<Self> {
    let expr : syn::ExprRange = input.parse()?;
    match expr.limits {
      syn::RangeLimits::HalfOpen(..) => {},
      syn::RangeLimits::Closed(dde) => {
        return Err(Error::new_spanned(dde, "Use '..' for range constraints"))
      },
    }
    Ok(GrbRangeExpr {lb: expr.from, ub: expr.to})
  }
}


struct RangeConstr {
  expr: syn::Expr,
  range: GrbRangeExpr,
}

impl Parse for RangeConstr {
  fn parse(input: ParseStream) -> Result<Self> {
    let expr = input.parse()?;
    input.parse::<Token![in]>()?;
    let range = input.parse()?;
    Ok(RangeConstr { expr, range })
  }
}

impl ToTokens for RangeConstr {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let expr = &self.expr;

    let lb = self.range.lb.as_ref().map(|lb| lb.to_token_stream()).unwrap_or(quote!{ -gurobi::INFINITY });
    let ub = self.range.ub.as_ref().map(|ub| ub.to_token_stream()).unwrap_or(quote!{ gurobi::INFINITY });

    let ts : TokenStream2 = quote!{
      gurobi::constr::RangeExpr{
        expr: gurobi::Expr::from(#expr),
        ub: #ub as f64,
        lb: #lb as f64,
      }
    };
    ts.to_tokens(tokens)
  }
}

enum ConstrExpr {
  Inequality(InequalityConstr),
  Range(RangeConstr)
}

impl Parse for ConstrExpr {
  fn parse(input: ParseStream) -> Result<Self> {
    // Forward-scan for the `in` keyword -- top level tokens only, don't walk the whole tree
    let in_found = {
      let mut curs = input.cursor();
      let in_  = Ident::new("in", Span::call_site());
      let mut in_found = false;
      while let Some((tt, next)) = curs.token_tree() {
        match tt {
          TokenTree::Ident(i) if i == in_ => {
            in_found = true;
            break;
          },
          _ => curs = next,
        }
      }
      in_found
    };

    if in_found {
      input.parse::<RangeConstr>().map(Range)
    } else {
      input.parse::<InequalityConstr>().map(Inequality)
    }
  }
}

impl ToTokens for ConstrExpr {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    match self {
      Inequality(e) => e.to_tokens(tokens),
      Range(e) => e.to_tokens(tokens),
    }
  }
}


/// A proc-macro for creating [`constr::ConstrExpr`] objects.
///
/// The syntax is `c!( lhs CMP rhs )` where `CMP` is one of: `==`, `>=` or `<=`.
#[proc_macro]
pub fn c(expr: TokenStream) -> TokenStream {
  let expr = syn::parse_macro_input!(expr as ConstrExpr);
  expr.into_token_stream().into()
}
