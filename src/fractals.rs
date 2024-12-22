// Fractals data structure and methods.

use log::info;

use image::{Rgb, RgbImage};
use num_complex::Complex;
use serde::Deserialize;
use std::f64::consts;
use std::fmt;
use std::time::{Instant, Duration};
use std::fs::{self};
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
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
    pub recentre_duration: Duration,
    pub image_filename: String,
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
            recentre_duration: Duration::new(0, 0),
            image_filename: String::from(""),
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
        self.escape_its = vec![vec![0; self.cols as usize]; self.rows as usize];
    }


    // Method to initialize fractal limits.
    // Need the fractal parameters initialised first.
    pub fn init_fractal_limits(&mut self) {
        info!("Initialising fractal limits.");
        self.left_lim = self.mid_pt.re - (self.cols as f64 / 2.0) * self.pt_div;
        self.top_lim = self.mid_pt.im + (self.rows as f64 / 2.0) * self.pt_div;
        self.pt_lt.re = self.left_lim;
        self.pt_lt.im = self.top_lim;      
    }

    pub fn init_col_pallete(&mut self) -> io::Result<()> {
        info!("Importing default colour palette.");

        // Read default palette from toml file.
        let toml_str = fs::read_to_string("./palettes/default.palette")?;
    
        // Deserialize into the Root struct
        let root: Root = toml::from_str(&toml_str).expect("Failed to deserialize palette.");
        
        // Map the palette entries into your desired format
        self.col_palette = root.palette
            .into_iter()
            .map(|entry| (entry.index, entry.color))
            .collect();
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

        // Render the image according to divergence calculations.
        self.render_image();

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

    // Method to recentre fractal image.
    pub fn recentre_fractal(&mut self, c_row: u32, c_col: u32) -> Result<(), FractalError> {
        info!("Recentreing fractal.");

        println!("Recentering to: ({:?}, {:?})", c_row, c_col);

        // Initialise timer for function.
        let recentre_start = Instant::now();

        // Report ok status and timing.
        self.recentre_duration = recentre_start.elapsed();
        info!("Time to recentre fractal: {:?}", self.generate_duration);

        Ok(())
    }

    // Function to render the image according to the
    // defined colour palette.
    pub fn render_image(&mut self) {
        info!("Rendering image according to colour palette.");

        // We need to remove the path from the filename,
        // as we are not interested in the original path.
        let raw_filename = &self.settings.fractal_filename;

        // Get file path for the file to be written.
        // All files will be written to a specific folder.
        let mut wrt_path = PathBuf::new();       
        wrt_path.push(&self.settings.fractal_folder);
        wrt_path.push(raw_filename);
        let mut wrt_path_string = wrt_path.to_string_lossy().into_owned();

        // Check if we are going to overwrite an existing file.
        // If so we will add a suffix to the end of the file name
        // to make it unique.
        let mut suffix = 1;
        let original_filename = wrt_path_string.clone();
        while Path::new(&wrt_path_string).exists() {
            // Construct next suffix.
            let extension = match original_filename.rfind('.') {
                Some(idx) => &original_filename[idx..],
                None => "",
            };
            // Construct base file path.
            let base_filename = if let Some(idx) = original_filename.rfind('.') {
                &original_filename[..idx]
            } else {
                &original_filename
            };
            // Construct complete file name.
            wrt_path_string = format!("{}-{:03}{}", base_filename, suffix, extension);
            // Increment suffix if this file name exists.
            suffix += 1;
        }

        // Define an image of the right size.
        let rows = self.rows;
        let cols = self.cols;
        let mut img = RgbImage::new(cols, rows);

        // Iterate through rows and columuns and
        // set the pixel colour accordingly.
        for y in 0..rows {
            for x in 0..cols{
                let pt_its: u32 = self.escape_its[y as usize][x as usize];
                let px_col: Rgb<u8> = det_px_col(pt_its, &self.col_palette);
                img.put_pixel(x, y, px_col);
            }
        }

        // Save the image.
        let _ = img.save(wrt_path_string.clone());

        // Save image filename without path for sending to file store.
        self.image_filename = wrt_path_string.clone();
        info!("Saving fractal image to: {:?}", wrt_path_string);
    }
}

// Function to determine the colour of the pixel.
// Based on linear interpolation of colour palette.
pub fn det_px_col(its: u32, col_pal: &Vec<(u32, (u8, u8, u8))>) -> Rgb<u8> {

    // Iterate through the boundaries to find where `its` fits
    // between consecutive boundaries.
    for i in 0..col_pal.len() - 1 {
        let (lower_bound, lower_color) = col_pal[i];
        let (upper_bound, upper_color) = col_pal[i + 1];

        if its > lower_bound && its <= upper_bound {
            // Perform linear interpolation between the two colours.
            let t = (its - lower_bound) as f32 / (upper_bound - lower_bound) as f32;
            let r = (1.0 - t) * lower_color.0 as f32 + t * upper_color.0 as f32;
            let g = (1.0 - t) * lower_color.1 as f32 + t * upper_color.1 as f32;
            let b = (1.0 - t) * lower_color.2 as f32 + t * upper_color.2 as f32;

            // Return interpolated colour for the pixel.
            return Rgb([r as u8, g as u8, b as u8]);
        }
    }

    // Handle the case where `its` doesn't fit into any range.
    // For simplicity, return the last colour in the palette.
    if let Some(&(last_bound, last_color)) = col_pal.last() {
        if its > last_bound {
            return Rgb([last_color.0, last_color.1, last_color.2]);
        }
    }

    // Default fallback colour (e.g., black).
    Rgb([0, 0, 0])
}
