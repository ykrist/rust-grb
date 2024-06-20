use grb::prelude::*;

#[test]
fn optimize() -> grb::Result<()> {
    let mut model = Model::new("model")?;

    let x = add_ctsvar!(model, name: "x", bounds: 0..)?;
    let y = add_ctsvar!(model, name: "y", bounds: 0..)?;

    model.add_constr("c", c!(x + y <= 10))?;

    model.set_objective(2 * x + 1, ModelSense::Maximize)?;
    model.optimize()?;
    let obj1 = model.get_attr(attr::ObjVal)?;

    let x1 = model.get_obj_attr(attr::X, &x)?;
    let y1 = model.get_obj_attr(attr::X, &y)?;
    println!("x:{x1}    y:{y1}");

    model.set_objective(y, ModelSense::Maximize)?;
    model.optimize()?;

    let obj2 = model.get_attr(attr::ObjVal)?;
    let x2 = model.get_obj_attr(attr::X, &x)?;
    let y2 = model.get_obj_attr(attr::X, &y)?;
    println!("x:{x2}    y:{y2}");

    assert_eq!(obj1, 21.);
    assert_eq!(x1, 10.);
    assert_eq!(y1, 0.);

    assert_eq!(obj2, 10.);
    assert_eq!(x2, 0.);
    assert_eq!(y2, 10.);

    Ok(())
}
