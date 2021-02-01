#[macro_export]
macro_rules! create_model {
  ($gag:ident, $model:ident $(,$var:ident)*) => {
    let $gag = gag::Gag::stdout().unwrap();
    let mut $model = Model::new("test")?;
    $(
      let $var = add_binvar!($model)?;
    )*
  }
}