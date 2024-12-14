// Fractals data structure and methods.

use log::info;

use num_complex::Complex;
use std::fmt;
use std::time::{Instant, Duration};

use crate::settings::Settings;
use crate::SETTINGS;

// Error result enum.
#[derive(Debug)]
pub enum FractalError {
    NotGenerated,
}

// Display of Fractal specific errors.
impl fmt::Display for FractalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            FractalError::NotGenerated => write!(f, "Failed to generate fractal image."),
        }
    }
}

// Struct of parameters for fractals generation.
pub struct Fractal {
    pub settings: Settings,
    pub rows: u32,
    pub cols: u32,
    pub mid_pt: Complex<f64>,
    pub pt_div: f64,
    pub max_its: u32,
    pub left_lim: f64,
    pub top_lim: f64,
    pub escape_its: Vec<Vec<u32>>,
    pub pt_lt: Complex<f64>,
    pub col_palete: Vec<(u32, (u8, u8, u8))>,
    pub generate_duration: Duration,
    pub calc_duration: Duration,
    pub render_duration: Duration,
}

// Initialise all struct variables.
// This method called at the start.
impl Fractal {
    pub fn init() -> Self {
        info!("Initialising Fractal struct.");

        // Lock the global SETTINGS to obtain access to the Settings object.
        let settings = SETTINGS.lock().unwrap().clone();

        Fractal {
            settings: settings,
            rows: 0,
            cols: 0,
            mid_pt: Complex::new(0.0, 0.0),
            pt_div: 0.0,
            max_its: 0,
            left_lim: 0.0,
            top_lim: 0.0,
            escape_its: Vec::new(),
            pt_lt: Complex::new(0.0, 0.0),
            col_palete: Vec::new(),
            generate_duration: Duration::new(0, 0),
            calc_duration: Duration::new(0, 0),
            render_duration: Duration::new(0, 0),
        }
    }

    // Method to initialize starting fractal image.
    pub fn init_fractal_image(&mut self) {
        info!("Initialising fractal parameters.");
        self.rows = self.settings.init_rows;
        self.cols = self.settings.init_cols;
        self.pt_div = self.settings.init_pt_div;
        let mid_pt_re:f64 = self.settings.init_mid_pt_re;
        let mid_pt_im:f64 = self.settings.init_mid_pt_im;
        self.mid_pt = Complex::new(mid_pt_re, mid_pt_im);
        self.max_its = self.settings.init_max_its;
        self.left_lim = self.mid_pt.re - (self.cols as f64 / 2.0) * self.pt_div;
        self.top_lim = self.mid_pt.im + (self.rows as f64 / 2.0) * self.pt_div;
        self.pt_lt.re = self.left_lim;
        self.pt_lt.im = self.top_lim;      
    }

    // Method to generate fractal image.
    pub fn generate_fractal(&mut self) -> Result<(), FractalError> {
        info!("Generating fractal.");

        // Initialise timer for function.
        let generate_start = Instant::now();

        // Generate fractal image.
        // TODO.
        // Return an error code if fail to generate, e.g.
        // return Err(FractalError::NotGenerated);

        // Report ok status and timing.
        self.generate_duration = generate_start.elapsed();
        info!("Time to generate fractal: {:?}", self.generate_duration);

        Ok(())
    }   
}
