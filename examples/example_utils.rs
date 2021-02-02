use grb::{Env, Model};

pub fn load_model_file_from_clarg(env: &Env) -> Model {
    let filepath = &std::env::args().nth(1).expect(
        "Binary requires an .lp file as argument (run one of the other examples first to obtain one)");
    Model::read_from(filepath, env).unwrap()
}
