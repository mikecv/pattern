// Fractals data structure and methods.

use log::info;

use num_complex::Complex;
use serde::Deserialize;
use std::f64::consts;
use std::fmt;
use std::time::{Instant, Duration};
use std::fs::{self};
use std::io::{self};
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

// Fractal colour palette formats.
#[derive(Debug, Deserialize)]
struct Root {
    palette: Vec<PaletteEntry>,
}

#[derive(Debug, Deserialize)]
struct PaletteEntry {
    index: u32,
    color: (u8, u8, u8),
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
    pub col_palette: Vec<(u32, (u8, u8, u8))>,
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
            col_palette: Vec::new(),
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
        self.escape_its = vec![vec![0; self.cols as usize]; self.rows as usize];
    }

    pub fn init_col_pallete(&mut self) -> io::Result<()> {
        info!("Importing default colour palette.");
    
        let toml_str = fs::read_to_string("./palettes/default.palette")?;
        println!("TOML: {:?}", toml_str);
    
        // Deserialize into the Root struct
        let root: Root = toml::from_str(&toml_str).expect("Failed to deserialize palette.");
        
        // Map the palette entries into your desired format
        self.col_palette = root.palette
            .into_iter()
            .map(|entry| (entry.index, entry.color))
            .collect();
    
        println!("CONFIG: {:?}", self.col_palette);
        Ok(())
    }

    // Method to generate fractal image.
    pub fn generate_fractal(&mut self) -> Result<(), FractalError> {
        info!("Generating fractal.");

        // Initialise timer for function.
        let generate_start = Instant::now();

        // Generate fractal image.
        // Return an error code if fail to generate, e.g.
        // return Err(FractalError::NotGenerated);

        // Start with the left top point.
        let mut st_c: Complex<f64> = self.pt_lt;

        // Iterate calculation over rows.
        for row in 0..self.rows {
            // Calculate the starting point for the row.
            // Just need to deduct incremental distance from
            // every row after the first (top) row.
            if row > 0 {
                st_c.im -= self.pt_div;
            }

            // Calculate divergence for row.
            self.cal_row_divergence(row, st_c);
        }

        // Report ok status and timing.
        self.generate_duration = generate_start.elapsed();
        info!("Time to generate fractal: {:?}", self.generate_duration);

        Ok(())
    }

    // Methed to calculate fractal divergence at a single point.
    // For points that reach the iteration count caculate
    // fractional divergence.
    pub fn cal_row_divergence(&mut self, row: u32, st_c: Complex<f64>) {
        // Iterante over all the columns in the row.
        // Starting point is left of the row.
        let mut pt_row: Complex<f64> = st_c;

        for col in 0..self.cols {
            // Iterate point along the row.
            if col > 0 {
                pt_row.re += self.pt_div;
            }

            // Define diverges flag and set to false.
            let mut diverges: bool = false;

            // Initialise divergence result to complex 0.
            let mut px_fn: Complex<f64> = Complex::new(0.0, 0.0);

            // Initialise number of iterations.
            let mut num_its: u32 = 1;

            // Keep iterating until function diverges.
            while !diverges && (num_its < self.max_its) {
                // Perform Mandelbrot function Fn+1 = Fn^2 + pt_row.
                px_fn = (px_fn * px_fn) + pt_row;
                // Check if function diverges.
                // Will diverge if modulus equal or greater than 2.
                if px_fn.norm() >= 2.0 {
                    diverges = true;
                }
                else {
                    num_its += 1;
                }
            }

            // Calculate fractional divergence for higher definition.
            let mod_fn = px_fn.norm();
            let mu_log = if mod_fn > consts::E {
                (mod_fn.ln().ln()) / consts::LN_2
            } else {
                0.0
            };
            let mut mu = num_its as f64 + 1.0 - mu_log;

            // Limit fractional divergence to maximum iterations
            if mu > self.max_its as f64 {
                mu = self.max_its as f64;
            }
            num_its = mu as u32;

            // Save number of iterations at the point point.
            self.escape_its[row as usize][col as usize] = num_its;
        }
    }
}
