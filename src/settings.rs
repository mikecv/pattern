use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub program_name: String,
    pub program_ver: String,
    pub program_devs: Vec<String>,
    pub program_web: String,
    pub fractal_folder: String,

    // Initial fractal setting.
    pub init_rows: u32,
    pub init_cols: u32,
    pub init_mid_pt_re: f64,
    pub init_mid_pt_im: f64,
    pub init_pt_div: f64,
    pub init_max_its: u32,
}
