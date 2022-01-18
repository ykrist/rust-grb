use grb::Model;

pub fn load_model_file_from_clarg() -> Model {
    let filepath = &std::env::args().nth(1).expect(
        "Binary requires an .lp file as argument (run one of the other examples first to obtain one)");
    Model::from_file(filepath).unwrap()
}
