// Fractals data structure and methods.

use log::info;
use log::debug;

use image::{Rgb, RgbImage};
use num_complex::Complex;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::json;
use std::f64::consts;
use std::fmt;
use std::time::{Instant, Duration};
use std::fs::{self};
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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
    rel_index: f32,
    index: u32,
    comment: String,
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
    pub col_palette: Vec<(f32, u32, String, (u8, u8, u8))>,
    pub generate_duration: Duration,
    pub recentre_duration: Duration,
    pub rendering_duration: Duration,
    pub histogram_duration: Duration,
    pub image_filename: String,
    pub histogram_data_json: String,
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
            rendering_duration: Duration::new(0, 0),
            histogram_duration: Duration::new(0, 0),
            image_filename: String::from(""),
            histogram_data_json: String::from(""),
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

        // Resize the escape iterations vector in case the
        // image size has been changed by the user,
        self.escape_its = vec![vec![0; self.cols as usize]; self.rows as usize];
    }

    pub fn init_col_pallete(&mut self) -> io::Result<()> {
        info!("Importing default colour palette.");

        // Create path to default palette file.
        let mut col_path = PathBuf::new();       
        col_path.push(&self.settings.palette_folder);
        col_path.push(&self.settings.def_palette);
        let col_path_string = col_path.to_string_lossy().into_owned();

        // Read default palette from toml file.
        let toml_str = fs::read_to_string(&col_path_string)?;

        // Deserialize into the Root struct
        let root: Root = toml::from_str(&toml_str).expect("Failed to deserialize palette.");
        
        // Map the palette entries into palette structure.
        self.col_palette = root.palette
            .into_iter()
            .map(|entry| (entry.rel_index, entry.index, entry.comment, entry.color))
            .collect();

        // Scale colour palette according to max iterations.
        for col_bound in 0..self.col_palette.len() {
            let (lower_rel_index, _lower_bound, _lower_comment, _lower_color) = &self.col_palette[col_bound];
            self.col_palette[col_bound].1 = (lower_rel_index * self.max_its as f32) as u32;
        }
        
        Ok(())
    }

    // Method to generate fractal image.
    pub fn generate_fractal(&mut self) -> Result<(), FractalError> {
        info!("Generating fractal.");

        // Initialise timer for function.
        let generate_start = Instant::now();
    
        // Wrap escape_its in an Arc<Mutex<_>> for thread-safe mutable access.
        let escape_its = Arc::new(Mutex::new(vec![vec![0; self.cols as usize]; self.rows as usize]));
    
        // Use parallel iteration over rows.
        (0..self.rows).into_par_iter().for_each(|row| {
            let mut st_c = self.pt_lt;
            st_c.im -= self.pt_div * row as f64;

            // Calculate divergence for row.
            let mut row_data = vec![0; self.cols as usize];
            self.cal_row_divergence(row as usize, st_c, &mut row_data);
    
            // Lock the Mutex to safely access and modify escape_its.
            let mut escape_its_locked = escape_its.lock().unwrap();
            escape_its_locked[row as usize] = row_data;
        });
    
        // After the parallel processing, escape_its can now be safely updated.   
        // Reassign the computed escape_its back to self.
        self.escape_its = Arc::try_unwrap(escape_its).unwrap().into_inner().unwrap();

        self.generate_duration = generate_start.elapsed();
        info!("Time to perform fractal divergence: {:?}", self.generate_duration);

        // Initialise timer for function.
        let rendering_start = Instant::now();  
    
        // Render the image according to divergence calculations.
        self.render_image();

        // Report ok status and timing.
        self.rendering_duration = rendering_start.elapsed();
        info!("Time to perform fractal rendering: {:?}", self.rendering_duration);

        Ok(())
    }
        
    // Methed to calculate fractal divergence at a single point.
    // For points that reach the iteration count calculate
    // fractional divergence.
    pub fn cal_row_divergence(&self, row: usize, st_c: Complex<f64>, row_data: &mut [u32]) {
        // Start divergence calculation timer for row.
        let start_time = Instant::now();

        // Point (col) in row for calculation.
        let mut pt_row = st_c;
    
        // Iterante over all the columns in the row.
        for col in 0..self.cols {
            if col > 0 {
                pt_row.re += self.pt_div;
            }

            // Define diverges flag and set to false.
            let mut diverges = false;

            // Initialise divergence result to complex 0.
            let mut px_fn = Complex::new(0.0, 0.0);

            // Initialise number of iterations.
            let mut num_its = 1;

            // Keep iterating until function diverges.
            while !diverges && (num_its < self.max_its) {
                px_fn = (px_fn * px_fn) + pt_row;
                if px_fn.norm() >= 2.0 {
                    diverges = true;
                } else {
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

            // Limit fractional divergence to maximum iterations'
            if mu > self.max_its as f64 {
                mu = self.max_its as f64;
            }
            row_data[col as usize] = mu as u32;
        }

        // Diagnostic time to do divergence processing on a particulat row.
        // This is only for checking on parallel processing.
        let duration = start_time.elapsed();
        debug!("Processed row {} in {:?}", row, duration);
    }
    
    // Method to recentre fractal image.
    pub fn recentre_fractal(&mut self, c_row: u32, c_col: u32) -> Result<(), FractalError> {
        info!("Recentring fractal to: (row:{:?}, col:{:?})", c_row, c_col);

        // Initialise timer for function.
        let recentre_start = Instant::now();

        // Re-initialise fractal parameters according to new
        // re-centred fractal.
        self.init_fractal_limits();

        // Generate fractal at recentred point.
        match self.generate_fractal() {
            Ok(_) => {
            
                // Report ok status and timing.
                self.recentre_duration = recentre_start.elapsed();
                info!("Time to recentre fractal: {:?}", self.generate_duration);

                Ok(())
            }
            Err(_e) => {
                // Fractal generation failed, respond with error.
                return Err(FractalError::NotGenerated);
            }
        }
    }

    // Function to render the image according to the
    // defined colour palette.
    pub fn render_image(&mut self) {
        info!("Rendering image according to colour palette.");

        // We need to remove the path from the filename,
        // as we are not interested in the original path.
        let raw_filename = &self.settings.fractal_filename;

        // Get the folder where files should be written.
        let wrt_path = PathBuf::from(&self.settings.fractal_folder);

        // Ensure the folder exists (optional, if folder creation is necessary).
        std::fs::create_dir_all(&wrt_path).expect("Failed to create fractal folder");

        // Extract base name and extension.
        // Includes the dot (e.g., ".png").
        let extension = match raw_filename.rfind('.') {
            Some(idx) => &raw_filename[idx..],
            None => "",
        };
        // Excludes the dot.
        let base_filename = match raw_filename.rfind('.') {
            Some(idx) => &raw_filename[..idx],
            None => raw_filename,
        };

        // Start with suffix 1 for filename `fractal-001`.
        let mut suffix = 1;
        let mut wrt_path_string;

        // Loop until we find a unique filename.
        loop {
            // Format the filename with the current suffix.
            let filename = format!("{}-{:03}{}", base_filename, suffix, extension);

            // Set the filename while keeping the directory path intact.
            let mut full_path = wrt_path.clone();
            full_path.push(filename);
            wrt_path_string = full_path.to_string_lossy().into_owned();

            // Break if the file does not exist.
            if !Path::new(&wrt_path_string).exists() {
                break;
            }

            // Increment suffix to try the next filename.
            suffix += 1;
        }

        // Define an image of the right size.
        let rows = self.rows;
        let cols = self.cols;
        let mut img = RgbImage::new(cols, rows);

        // Iterate through rows and columns and
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

    // Method to generate a divergence histogram chart image.
    pub fn divergence_histogram(&mut self) -> Result<(), FractalError> {
        info!("Generating fractal divergence histogram.");

        // Initialise timer for function.
        let histogram_start = Instant::now();

        // Initialise plot vector.
        let mut data = Vec::new();

        // Iterate through possible iteration counts, i.e. 1 to maximum iterations.
        // Then find number of occurances of its in the fractal image pixel divergence count,
        // which is also going to be from 1 to maximum iterations.
        // So the plot x axis is 1 to max_its, and y axis will be 1 to (rows x columns) worst case.
        // Keep track of maximum interations count (max_count).
        // And the largest iteration encountered (end_its).
        // This is used to limit the axis lengths.
        let mut max_count: u32 = 0;
        let mut _end_its: u32 = 1;

        // Iterate through all possible iteration counts.
        for its in 0..self.max_its {
                let mut its_cnt: u32 = 0;

            // Iterate through all pixels and check for matched iteration count.
            for y in 0..self.cols {
                for x in 0..self.rows {
                    // Check if the iterations count matches divergence count for this pixel.
                    if self.escape_its[x as usize][y as usize] == its {
                        its_cnt += 1;
                        // Check if new maximum.
                        if its_cnt > max_count {
                            max_count = its_cnt;
                        }
                    }
                }
            }

            // Check if any divergence at this count.
            // If so, it's the largest count so far.
            if its_cnt > 0 {
                _end_its = its;
            }

            // Push the iteration and count to the data array.
            data.push((its, its_cnt));
        }

        // Assemble histogram data from the 'data' vector in the format
        // of bins and counts as follows:
        //
        // self.histogram_data_json = json!({
        //     "bins": [0, 1, 2, 3, 4, 5, 6],
        //     "counts": [10, 20, 30, 25, 15, 17, 2]
        // }).to_string();

        self.histogram_data_json = json!({
            "bins": data.iter().map(|&(its, _)| its).collect::<Vec<u32>>(),
            "counts": data.iter().map(|&(_, count)| count).collect::<Vec<u32>>()
        }).to_string();

        // Report ok status and timing.
        self.histogram_duration = histogram_start.elapsed();
        info!("Time to generate divergence histogram: {:?}", self.histogram_duration);

        Ok(())
    }  
}

// Function to determine the colour of the pixel.
// Based on linear interpolation of colour palette.
pub fn det_px_col(its: u32, col_pal: &Vec<(f32, u32, String, (u8, u8, u8))>) -> Rgb<u8> {

    // Iterate through the boundaries to find where `its` fits
    // between consecutive boundaries.
    for i in 0..col_pal.len() - 1 {
        let (_lower_rel_index, lower_bound, _lower_comment, lower_color) = &col_pal[i];
        let (_upper_rel_index, upper_bound, _upper_comment, upper_color) = &col_pal[i + 1];

        if its > *lower_bound && its <= *upper_bound {
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
    if let Some(&(_last_rel, last_bound, ref _last_comment, last_color)) = col_pal.last() {
        if its > last_bound {
            return Rgb([last_color.0, last_color.1, last_color.2]);
        }
    }

    // Default fallback colour (e.g., black).
    Rgb([0, 0, 0])
}
