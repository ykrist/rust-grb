use proc_macro2::{TokenStream as TokenStream2, TokenTree, Ident, Span};
use quote::{ToTokens, quote, quote_spanned, TokenStreamExt};
use syn::{Token, Result, Error, Expr};
use syn::parse::{ParseStream, Parse};
use syn::spanned::Spanned;

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
      Eq(..) => quote! { grb::ConstrSense::Equal },
      Le(..) => quote! { grb::ConstrSense::Less },
      Ge(..) => quote! { grb::ConstrSense::Greater },
      Lt(..) | Gt(..) | Ne(..) => { return Err(Error::new_spanned(cmpexpr.op, "expected >=, <= or ==")); }
      _ => { return Err(Error::new_spanned(cmpexpr, "expression should be a ==, >= or <= comparison")); }
    };

    Ok(InequalityConstr {lhs: cmpexpr.left, sense, rhs:cmpexpr.right})
    }
}

impl ToTokens for InequalityConstr {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    // let Self{ ref lhs, ref rhs, ref sense } = self;
    let lhs = self.lhs.as_ref();
    let lhs = quote_spanned!{ lhs.span()=> grb::Expr::from(#lhs) };
    let rhs = self.rhs.as_ref();
    let rhs = quote_spanned!{ rhs.span()=> grb::Expr::from(#rhs) };
    let sense = &self.sense;
    let ts = quote! {
      grb::constr::IneqExpr{
        lhs: #lhs,
        sense: #sense,
        rhs: #rhs,
      }
    };
    ts.to_tokens(tokens);
  }
}

#[derive(Debug, Default, Clone)]
struct GrbRangeExpr {
  lb: Option<Box<syn::Expr>>,
  ub: Option<Box<syn::Expr>>,
}

impl GrbRangeExpr {
  pub fn ub_to_tokens(&self) -> TokenStream2 {
    match self.ub {
      Some(ref x) => quote_spanned!{ x.span()=>  #x as f64},
      None => quote!{ grb::INFINITY },
    }
  }

  pub fn lb_to_tokens(&self) -> TokenStream2 {
    match self.lb {
      Some(ref x) => quote_spanned!{ x.span()=> #x as f64},
      None => quote!{ -grb::INFINITY },
    }
  }
}

impl Parse for GrbRangeExpr {
  fn parse(input: ParseStream) -> Result<Self> {
    let expr : syn::ExprRange = input.parse()?;
    match expr.limits {
      syn::RangeLimits::HalfOpen(..) => {},
      syn::RangeLimits::Closed(dde) => {
        return Err(Error::new_spanned(dde, "Use '..' for bounds and range constraints"))
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
    let expr = quote_spanned! { expr.span() => grb::Expr::from(#expr) };

    let lb = self.range.lb_to_tokens();
    let ub = self.range.ub_to_tokens();

    let ts : TokenStream2 = quote!{
      grb::constr::RangeExpr{
        expr: #expr,
        ub: #ub,
        lb: #lb,
      }
    };
    ts.to_tokens(tokens)
  }
}

#[allow(clippy::large_enum_variant)]
enum ConstrExpr {
  Inequality(InequalityConstr),
  Range(RangeConstr)
}

impl Parse for ConstrExpr {
  fn parse(input: ParseStream) -> Result<Self> {
    // Forward-scan for the `in` keyword -- top level tokens only, don't walk the whole tree
    // Heuristic that is more efficient than speculative parsing, and gives better error messages
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
      input.parse::<RangeConstr>().map(ConstrExpr::Range)
    } else {
      input.parse::<InequalityConstr>().map(ConstrExpr::Inequality)
    }
  }
}

impl ToTokens for ConstrExpr {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    match self {
      ConstrExpr::Inequality(e) => e.to_tokens(tokens),
      ConstrExpr::Range(e) => e.to_tokens(tokens),
    }
  }
}


/// A proc-macro for creating constraint objects.
///
/// # Syntax
/// ## Inequality constraints
/// To create an `IneqExpr` object for a linear or quadratic constraint, the syntax is
/// ```text
/// c!( LHS CMP RHS )
/// ```
/// `LHS` and `RHS` should be valid algebraic expressions involving `Var` objects and numeric constants.
/// For example, if `x`, `y` and `z` are `Var` objects and `vars` is an `Vec<Var>` objects, these are valid:
/// ```
/// # use grb::prelude::*;
/// # fn f(x: Var, y: Var, z: Var, vars: Vec<Var>){
///   c!(vars.iter().grb_sum() == x );
///   c!( x + 1/2 == 1.4*y - 2*z );
///   c!( 2*x >= z*y );
///   c!( 2*x >= 7*(z*y) ); // note the brackets on the non-linear term when a coefficient is present
/// # }
/// ```
/// but the following are not:
/// ```compile_fail
/// # use grb::*;
/// # fn f(x: Var, y: Var, z: Var){
///   c!(vars.iter().sum() == x ); // cannot infer type on sum() call
///   c!( 2*x >= z >= y ); // chained comparison
///   c!( 2*x >= 7*z*y ); // no brackets around var*var when a coefficient is present
/// # }
/// ```
/// The macro expands `c!( LHS == RHS )` to:
/// ```
/// # let LHS = 0;
/// # let RHS = 0;
/// grb::constr::IneqExpr {
///   lhs: grb::Expr::from(LHS),
///   sense: grb::ConstrSense::Equal,
///   rhs: grb::Expr::from(RHS),
/// };
/// ```
///
/// ## Range constraints
/// To create a `RangeExpr` object for a range constraint, use the syntax
/// ```text
/// c!( EXPR in LB..UB )
/// c!( EXPR in LB.. )
/// c!( EXPR in ..UB )
/// c!( EXPR in .. )
/// ```
/// where `EXPR` is a valid expression, like `LHS` and `RHS` above.  Additionally, `EXPR` must be linear,
/// although this is not checked at compile-time.
///
/// `LB` and `UB` can be any expression that evaluates to type that can be cast to a `f64` using
/// the `as` operator. For example, the following are valid (variables have the meaning as above):
/// ```
/// # use grb::prelude::*;
/// # fn f(x: Var, y: Var, z: Var, vars: Vec<Var>){
///   c!( x - y + 2*z in 0..200 );
///   c!( x - y + 2*z in 1.. );
///   c!( x - y in (1.0/3.0)..(1<<4));
/// # }
/// ```
///
#[proc_macro]
pub fn c(expr: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let expr = syn::parse_macro_input!(expr as ConstrExpr);
  expr.into_token_stream().into()
}

trait OptionalArg {
  type Value: Parse;
  fn name() -> &'static str;
  fn value(&self) -> &Option<Self::Value>;
  fn value_mut(&mut self) -> &mut Option<Self::Value>;

  fn match_parse(&mut self, name: &syn::Ident, input: &ParseStream) -> Result<bool> {
    if name == Self::name() {
      input.parse::<Token![:]>()?;
      let v = self.value_mut();
      if v.is_some() { return Err(Error::new_spanned(name, "duplicate argument"))}
      *v = Some(input.parse()?);
      Ok(true)
    } else {
      Ok(false)
    }
  }
}

trait OptionalArgDefault: OptionalArg {
  fn default_value() -> TokenStream2;
}

macro_rules! impl_optional_arg {
    ($t:ident, $vt:path, $name:expr, $default:expr) => {
      impl_optional_arg!{$t, $vt, $name}

      impl OptionalArgDefault for $t {
        fn default_value() -> TokenStream2 { $default }
      }

      impl ToTokens for $t {
        fn to_tokens(&self, tokens: &mut TokenStream2) {
          match self.value() {
            None => tokens.append_all(Self::default_value()),
            Some(v) => v.to_tokens(tokens),
          }
        }
      }

    };

    ($t:ident, $vt:path, $name:expr) => {

      #[derive(Debug)]
      struct $t(Option<$vt>);

      impl OptionalArg for $t {
      type Value = $vt;
        fn name() -> &'static str { $name }

        fn value(&self) -> &Option<$vt> { &self.0 }
        fn value_mut(&mut self) -> &mut Option<$vt> { &mut self.0 }
      }
    };
}

impl_optional_arg!(VarName, syn::Expr, "name", quote!{ "" });
impl_optional_arg!(VarObj, syn::Expr, "obj", quote!{ 0.0 });
impl_optional_arg!(VarBounds, GrbRangeExpr, "bounds");

struct OptArgs {
  name: VarName,
  obj: VarObj,
  bounds: VarBounds,
}

impl OptArgs {
  pub fn to_token_stream(&self, model: &syn::Ident, vtype: &impl ToTokens) -> TokenStream2 {
    let name = &self.name;
    let obj = &self.obj;
    let (lb, ub) = match self.bounds.0 {
      Some(ref bounds) => (bounds.lb_to_tokens(), bounds.ub_to_tokens()),
      None => (quote!{ 0.0f64 }, quote!{ grb::INFINITY })
    };

    quote!{ #model.add_var(#name, #vtype, #obj as f64, #lb, #ub, std::iter::empty() ) }
  }
}

impl Parse for OptArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut name = VarName(None);
    let mut bounds = VarBounds(None);
    let mut obj = VarObj(None);

    while !input.is_empty() {
      let comma = input.parse::<Token![,]>()?;
      let optname: syn::Ident = input.parse().map_err(|e| {
        if input.is_empty() {
          Error::new_spanned(comma, "unexpected end of input: remove trailing comma")
        } else {
          e
        }
      })?;

      if !(name.match_parse(&optname, &input)?
        || obj.match_parse(&optname, &input)?
        || bounds.match_parse(&optname, &input)?) {
        return Err(Error::new_spanned(&optname, format_args!("unknown argument '{}'", &optname)))
      };

    }
    Ok(OptArgs{ name, obj, bounds})
  }
}


struct AddVarInput {
  model: syn::Ident,
  vtype: syn::ExprPath,
  optargs : OptArgs,
}

impl Parse for AddVarInput {
  fn parse(input: ParseStream) -> Result<Self> {
    let model: syn::Ident = input.parse()?;
    input.parse::<Token![,]>()
      .map_err(|e| Error::new(e.span(), "expected `,` (macro expects 2 positional args)"))?;
    let vtype: syn::ExprPath = input.parse()?;
    let optargs = input.parse()?;
    Ok(AddVarInput { model, vtype, optargs })
  }
}

impl ToTokens for AddVarInput {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let out = self.optargs.to_token_stream(&self.model, &self.vtype);
    out.to_tokens(tokens);
  }
}


macro_rules! specialised_addvar {
    ($t:ident, $vtype:expr, $procmacroname:ident, $docvtype:expr) => {
      struct $t {
        model: syn::Ident,
        optargs : OptArgs,
      }

      impl Parse for $t {
        fn parse(input: ParseStream) -> Result<Self> {
          let model= input.parse()?;
          let optargs = input.parse()?;
          Ok(Self { model, optargs })
        }
      }

      impl ToTokens for $t {
        fn to_tokens(&self, tokens: &mut TokenStream2) {
          let vtype = $vtype;
          let out = self.optargs.to_token_stream(&self.model, &vtype);
          out.to_tokens(tokens);
        }
      }

      #[doc= "Equivalent to calling [`add_var`]`(model, "]
      #[doc= $docvtype]
      #[doc=", ...)`.\n\n"]
      #[proc_macro]
      pub fn $procmacroname(expr: proc_macro::TokenStream) -> proc_macro::TokenStream {
        syn::parse_macro_input!(expr as $t).into_token_stream().into()
      }
    };
}

specialised_addvar!(AddBinVarInput, quote!{ grb::VarType::Binary }, add_binvar, "Binary");
specialised_addvar!(AddCtsVarInput, quote!{ grb::VarType::Continuous }, add_ctsvar, "Continuous");
specialised_addvar!(AddIntVarInput, quote!{ grb::VarType::Integer }, add_intvar, "Integer");


/// Convienence wrapper around [`Model::add_var`] Add a new variable to a `Model` object.  The macro keyword arguments are
/// optional.
///
/// # Syntax
/// The syntax of macro is two positional arguments followed by any number of named arguments:
/// ```text
/// add_var!(MODEL, VAR_TYPE, NAMED_ARG1: VAL1, NAMED_ARG2: VAL2, ...)
/// ```
/// `MODEL` should be an instance of a `Model`.
///
/// `VAR_TYPE` should be the variable type - a variant of [`VarType`].
///
/// The named arguments are described below.
///
/// | Name     | Type                                                      | `Model::add_var` argument |
/// | -------- | -------------------------------------------------------   | --------------------------- |
/// | `name`   | Anything that implements `AsRef<str>` (&str, String, etc) | `name`                      |
/// | `obj`    | Anything that can be cast to a `f64`                      | `obj`                       |
/// | `bounds` | A range expression, see below                             | `ub` & `lb`                 |
///
/// The `bounds` argument takes a value of the form `LB..UB` where `LB` and `UB` are the upper and lower bounds of the variable.
///  `LB` and `UB` can be   left off as well, so `..UB` (short for `-INFINITY..UB`), `LB..` (short for `LB..INFINITY`) and `..`
/// are also valid values.
///
///
///
/// [`Model::add_var`]: struct.Model.html#method.add_var
/// [`VarType`]: enum.VarType.html
/// ```
/// use grb::prelude::*;
/// let mut model = Model::new("Model").unwrap();
/// add_var!(model, Continuous, name: "name", obj: 0.0, bounds: -10..10)?;
/// add_var!(model, Integer, bounds: 0..)?;
/// add_var!(model, Continuous, name: &format!("X[{}]", 42))?;
/// # Ok::<(), grb::Error>(())
/// ```
///
#[proc_macro]
pub fn add_var(expr: proc_macro::TokenStream) -> proc_macro::TokenStream {
  syn::parse_macro_input!(expr as AddVarInput).into_token_stream().into()
}
