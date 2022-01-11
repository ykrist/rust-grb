use grb::*;

mod example_utils;
use example_utils::*;

fn main() {
    let mut env = Env::new("tune.log").unwrap();
    // set the number of improved parameter sets.
    env.set(param::TuneResults, 1).unwrap();

    let mut model = load_model_file_from_clarg();

    model.tune().unwrap();

    let tune_cnt = model.get_attr(attr::TuneResultCount).unwrap();
    if tune_cnt > 0 {
        model.get_tune_result(0).unwrap();
        model.write("tune.prm").unwrap();

        model.optimize().unwrap();
    }
}
