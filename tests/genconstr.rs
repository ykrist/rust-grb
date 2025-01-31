use grb::prelude::*;

#[test]
fn main() -> anyhow::Result<()> {
    const N_LITERALS: usize = 4;
    const N_CLAUSES: usize = 8;
    const N_OBJ: usize = 2;

    let mut model = Model::new("genconstr")?;

    let pos: Vec<_> = (0..N_LITERALS)
        .map(|i| add_binvar!(model, name: &format!("X{i}")))
        .collect::<Result<_, _>>()?;

    let neg: Vec<_> = (0..N_LITERALS)
        .map(|i| add_binvar!(model, name: &format!("notX{i}")))
        .collect::<Result<_, _>>()?;

    let clauses: Vec<_> = (0..N_CLAUSES)
        .map(|i| add_binvar!(model, name: &format!("Clause{i}")))
        .collect::<Result<_, _>>()?;

    let objectives: Vec<_> = (0..N_OBJ)
        .map(|i| add_binvar!(model, name: &format!("Obj{i}"), obj: 1.0))
        .collect::<Result<_, _>>()?;

    for i in 0..N_LITERALS {
        model.add_constr(&format!("CNSTR_X{i}"), c!(pos[i] + neg[i] == 1))?;
    }

    let clauses_array: [[Var; 3]; N_CLAUSES] = [
        [pos[0], neg[1], pos[2]],
        [pos[1], neg[2], pos[3]],
        [pos[2], neg[3], pos[0]],
        [pos[3], neg[0], pos[1]],
        [neg[0], neg[1], pos[2]],
        [neg[1], neg[2], pos[3]],
        [neg[2], neg[3], pos[0]],
        [neg[3], neg[0], pos[1]],
    ];

    for i in 0..N_CLAUSES {
        model.add_genconstr_or(&format!("CNSTR_Clause{i}"), clauses[i], clauses_array[i])?;
    }

    model.add_genconstr_min("CNSTR_Obj0", objectives[0], clauses.clone(), None)?;

    model.add_genconstr_indicator(
        "CNSSTR_Obj1",
        objectives[1],
        true,
        c!(clauses.iter().grb_sum() >= 4),
    )?;

    model.set_attr(attr::ModelSense, Maximize)?;

    model.optimize()?;

    match model.status()? {
        Status::Optimal => {}
        Status::Infeasible | Status::InfOrUnbd | Status::Unbounded => {
            println!("The model cannot be solved \nbecause it is infeasible or unbounded")
        }
        s => println!("Optimization was stopped with status {s:#?}"),
    }

    let objval = model.get_attr(attr::ObjVal)?;

    if objval > 1.9 {
        println!("Logical expression is satisfiable");
    } else if objval > 0.9 {
        println!("At least four clauses can be satsfied");
    } else {
        println!("At most three clauses may be satisfied");
    }

    Ok(())
}
