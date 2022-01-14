use grb::prelude::*;

#[test]
fn main() -> anyhow::Result<()> {
    let mut m = Model::new("multi_scenario")?;

    let x = add_binvar!(m)?;
    let y = add_binvar!(m)?;

    let c1 = m.add_constr("c1", c!(x + y <= 2))?;
    

    m.set_attr(attr::NumScenarios, 3)?;

    m.set_param(param::ScenarioNumber, 1)?;
    m.set_obj_attr(attr::ScenNObj, &x, -1.0)?;
    m.set_obj_attr(attr::ScenNObj, &y, -1.0)?;
    m.set_obj_attr(attr::ScenNRHS, &c1, 1.0)?;
    

    m.set_param(param::ScenarioNumber, 2)?;
    m.set_obj_attr(attr::ScenNObj, &y, 1.0)?;
    m.set_obj_attr(attr::ScenNLB, &y, 1.0)?;
    
    m.optimize()?;
    

    for (idx, &correct_obj) in [0.0, -1.0, 1.0].iter().enumerate() {
        println!("--------------------------------------------------------------------");
        let idx = idx as i32;
        m.set_param(param::ScenarioNumber, idx)?;
        let obj = m.get_attr(attr::ScenNObjVal)?;
        let mut sm = m.single_scenario_model()?;
        sm.optimize()?;
        let obj2 = sm.get_attr(attr::ObjVal)?;
        assert_eq!(obj.round(), correct_obj);
        assert_eq!(obj2.round(), correct_obj);
    }

    Ok(())
}