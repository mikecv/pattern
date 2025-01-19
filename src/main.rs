// Fractals application.

use log::info;
use log4rs;

use actix_files as fsx;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{get, post, web, App, HttpRequest, HttpServer, HttpResponse, Responder};
use futures_util::stream::StreamExt;
use lazy_static::lazy_static;
use num_complex::Complex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::settings::Settings;
use crate::fractals::Fractal;

pub mod settings;
pub mod fractals;

// Create a global variable for application settings.
// This will be available in other files.
lazy_static! {
    static ref SETTINGS: Mutex<Settings> = {
        // Read YAML settings file.
        let mut file = futures::executor::block_on(File::open("settings.yml")).expect("Unable to open file");
        let mut contents = String::new();
        futures::executor::block_on(file.read_to_string(&mut contents)).expect("Unable to read file");

        // Deserialize YAML into Settings struct.
        let settings: Settings = serde_yaml::from_str(&contents).expect("Unable to parse YAML");
        Mutex::new(settings)
    };
}

// File server.
async fn serve_image(_req: HttpRequest, path: web::Path<String>) -> actix_web::Result<NamedFile> {
    let file_path = format!("./fractals/{}", path.into_inner());
    Ok(NamedFile::open(file_path)?)
}

// Application start (index) endpoint.
#[get("/")]
async fn intro() -> impl Responder {
    info!("Invoking UI application start endpoint.");

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();

    // Assign parameter defaults so that they can be passed to the UI.
    let default_rows = settings.init_rows;
    let default_cols = settings.init_cols;
    let default_centre_re = settings.init_mid_pt_re;
    let default_centre_im = settings.init_mid_pt_im;
    let default_division = settings.init_pt_div;
    let default_max_its = settings.init_max_its;

    // In the UI replace the parameter tags with the default values.
    // These will appear as pre-filled parameters in the ui.
    let html = include_str!("../static/index.html")
        .replace("{{ default_rows }}", &default_rows.to_string())
        .replace("{{ default_cols }}", &default_cols.to_string())
        .replace("{{ default_centre_re }}", &default_centre_re.to_string())
        .replace("{{ default_centre_im }}", &default_centre_im.to_string())
        .replace("{{ default_division }}", &default_division.to_string())
        .replace("{{ default_max_its }}", &default_max_its.to_string());

        HttpResponse::Ok().content_type("text/html").body(html)
}

// Define structure for fractal parameters payload.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct FractalParams {
    value1: Option<u32>,
    value2: Option<u32>,
    value3: Option<f64>,
    value4: Option<f64>,
    value5: Option<f64>,
    value6: Option<u32>,
    value7: Option<String>,
}

// Define structure for fractal recentre payload.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct FractalCentre {
    centre_col: u32,
    centre_row: u32,
    new_centre_re: f64,
    new_centre_im: f64,
}

// Define structure for fractal render payload.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct FractalParamsClear {
    rows: u32,
    cols: u32,
    centre_re: f64,
    centre_im: f64,
    pt_div: f64,
    max_its: u32,
}

// Generate fractal image endpoint.
#[post("/generate")]
async fn generate(fractal_params: web::Json<FractalParams>, fractal: web::Data<Arc<Mutex<Fractal>>>,) -> impl Responder {
    info!("Invoking fractal generation endpoint.");

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();

    // Get access to steg instance.
    let mut fractal = fractal.lock().unwrap();

    // Initialise fractal image.
    fractal.init_fractal_image();

    // Access parameters, note that these may have been edited in the UI.
    // Check if any parameters are set to none type,
    // if so, set to default setting.
    let mut params = fractal_params.into_inner();

    // Parameter 1.
    params.value1 = Some(params.value1.unwrap_or(settings.init_rows));
    fractal.rows = params.value1.unwrap();

    // Parameter 2.
    params.value2 = Some(params.value2.unwrap_or(settings.init_cols));
    fractal.cols = params.value2.unwrap();

    // Parameter 3 & 4.
    params.value3 = Some(params.value3.unwrap_or(settings.init_mid_pt_re));
    let mid_pt_re = params.value3.unwrap();
    params.value4 = Some(params.value4.unwrap_or(settings.init_mid_pt_im));
    let mid_pt_im = params.value4.unwrap(); 
    fractal.mid_pt = Complex::new(mid_pt_re, mid_pt_im);

    // Parameter 5.
    params.value5 = Some(params.value5.unwrap_or(settings.init_pt_div));
    fractal.pt_div = params.value5.unwrap();

    // Parameter 6.
    params.value6 = Some(params.value6.unwrap_or(settings.init_max_its));
    fractal.max_its = params.value6.unwrap();

    // Pass the currently active colour palette file (only the filename is required).
    // Only need this initially for the active palette file.
    params.value7 = Some(params.value7.unwrap_or_else(|| {
        Path::new(&fractal.active_palette_file)
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
            .to_string()
    }));

    // Initialise fractal limits.
    // Require the fractal parameters to be initialised beforehand.
    fractal.init_fractal_limits();

    // Initialise the colour palette as it may have changed.
    let _ = fractal.init_col_pallete();

    // Generate the fractal image.
    // Report status and payload to the front end.
    match fractal.generate_fractal() {
        Ok(_) => {
            let gen_time_ms:f64 = fractal.generate_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", gen_time_ms);

            // Ensure only the filename (not path) is sent to the frontend.
            let image_filename = std::path::Path::new(&fractal.image_filename)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let response_data = json!({
                "generation": "True",
                "time": duration_str,
                "error": "Success",
                "params": params,
                "image": image_filename,
            });

             // Respond with status to display on UI.
             HttpResponse::Ok().json(response_data)
        }
        Err(e) => {
            // Fractal generation failed, respond with error.
            let gen_time_ms:f64 = fractal.generate_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", gen_time_ms);

            let response_data = json!({
                "generation": "False",
                "time": duration_str,
                "error": e.to_string(),
                "params": params,
            });

             // Respond with status to display on UI.
             HttpResponse::InternalServerError().json(response_data)
        }
    }
}

// This moves the centre of the fractal and then generates the new fractal image.
// This could involve (but doesn't) copying parts of the already rendered
// fractal instead of performing divergence calculations on the whole image.
#[post("/recentre")]
async fn recentre(fractal_centre: web::Json<FractalCentre>, fractal: web::Data<Arc<Mutex<Fractal>>>,) -> impl Responder {
    info!("Invoking fractal recentre endpoint.");

    // Get application settings in scope.
    // Currently not used.
    let _settings: Settings = SETTINGS.lock().unwrap().clone();

    // Get access to steg instance.
    let mut fractal = fractal.lock().unwrap();

    // Access fractal new centre point.
    // New centre point row/col is just used to potentially speeding up
    // the generation by only generating new pixels (not moved ones).
    let centre_point = fractal_centre.into_inner();
    let centre_row = centre_point.centre_row;
    let centre_col = centre_point.centre_col;

    // Also set new real/imaginary coordinates of centre point for
    // generating new panned fractal image.
    let mid_pt_re:f64 = centre_point.new_centre_re;
    let mid_pt_im:f64 = centre_point.new_centre_im;
    fractal.mid_pt = Complex::new(mid_pt_re, mid_pt_im);
    info!("Recentring to x:{:?} y:{:?}", mid_pt_re, mid_pt_im);

    // Initialise colour palette as it may have changed.
    let _ = fractal.init_col_pallete();

    // Recentre and generate the fractal.
    // Report status and payload to front end.
    match fractal.recentre_fractal(centre_row, centre_col) {
        Ok(_) => {
            let pan_time_ms:f64 = fractal.generate_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", pan_time_ms);

            // Ensure only the filename (not path) is sent to the frontend.
            let image_filename = std::path::Path::new(&fractal.image_filename)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let response_data = json!({
                "recentred": "True",
                "time": duration_str,
                "error": "Success",
                "image": image_filename,
            });

             // Respond with status to display on UI.
             HttpResponse::Ok().json(response_data)
        }
        Err(e) => {
            // Fractal recentre and generation failed, respond with error.
            let pan_time_ms:f64 = fractal.generate_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", pan_time_ms);

            let response_data = json!({
                "recentred": "False",
                "time": duration_str,
                "error": e.to_string(),
            });

             // Respond with status to display on UI.
             HttpResponse::InternalServerError().json(response_data)
        }
    }
}

// Generate a histogram curve plot of iteration divergence count versus
// iteration count.
// This can be useful to determine the colour boundaries of colour
// render palettes.
#[get("/histogram")]
async fn histogram(fractal: web::Data<Arc<Mutex<Fractal>>>,) -> impl Responder {
    info!("Invoking divergence histogram endpoint.");

    // Get application settings in scope.
    // Currently not used.
    let _settings: Settings = SETTINGS.lock().unwrap().clone();

    // Get access to steg instance.
    let mut fractal = fractal.lock().unwrap();

    // Generate divergence histogram chart json data.
    // Report status and payload to front end.
    match fractal.divergence_histogram() {
        Ok(_) => {
            let hist_time_ms:f64 = fractal.histogram_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", hist_time_ms);

            let response_data = json!({
                "histogram": "True",
                "time": duration_str,
                "error": "Success",
                "chart": fractal.histogram_data_json,
            });

             // Respond with status to display on UI.
             HttpResponse::Ok().json(response_data)
        }
        Err(e) => {
            // Divergence histogram chart data generation failed, respond with error.
            let hist_time_ms:f64 = fractal.histogram_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", hist_time_ms);

            let response_data = json!({
                "histogram": "False",
                "time": duration_str,
                "error": e.to_string(),
                "chart": "",
            });

             // Respond with status to display on UI.
             HttpResponse::InternalServerError().json(response_data)
        }
    }
}

// Load a colour palette file and make it currently active.
// By default colour palette files are stored in a standard folder.
#[post("/palette")]
async fn palette(mut payload: Multipart, fractal: web::Data<Arc<Mutex<Fractal>>>,) -> impl Responder {
    info!("Invoking active colour palette endpoint.");

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();

    // Get access to steg instance.
    let mut fractal = fractal.lock().unwrap();

    // Ensure that the directory exists.
    // The colour palette folder is stored in settings.
    fs::create_dir_all(&settings.palette_folder).unwrap_or_default();

    let mut active_palette_file = None;

    while let Some(field) = payload.next().await {
        if let Ok(mut field) = field {
            if let Some(content_disposition) = field.content_disposition().cloned() {
                if let Some(filename) = content_disposition.get_filename() {
                    let filepath = format!("{}/{}", &settings.palette_folder, filename);
            
                    // Save the palette file, overwrite it if necessary.
                    let mut f = match fs::File::create(filepath.clone()) {
                        Ok(file) => file,
                        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
                    };
            
                    while let Some(chunk) = field.next().await {
                        match chunk {
                            Ok(data) => {
                                if f.write_all(&data).is_err() {
                                    return HttpResponse::InternalServerError().body("File write error");
                                }
                            }
                            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
                        }
                    }
            
                    active_palette_file = Some(filename.to_string());
                }
            }
        }
    }
  
    if let Some(palette_file) = active_palette_file {
        // Update the fractal instance with the new active palette file.
        fractal.active_palette_file = palette_file.clone();

        // Construct payload for UI.
        let response_data = json!({
            "palette": "True",
            "palette_file": palette_file,
        });

        HttpResponse::Ok().json(response_data)
    } else {
        HttpResponse::BadRequest().body("No palette file provided")
    }
}

// Performs rendering of the current fractal image according to the
// current active colour palette.
// Note that the UI render button triggering this endpoint is disabled
// until the fractal image has first been generated.
// Note also that the initial fractal image is rendered using
// the default colour palette.
#[post("/render")]
async fn render(fractal: web::Data<Arc<Mutex<Fractal>>>,) -> impl Responder {
    info!("Invoking fractal re-render endpoint.");

    // Get application settings in scope.
    // Currently not used.
    let _settings: Settings = SETTINGS.lock().unwrap().clone();

    // Get access to steg instance.
    let mut fractal = fractal.lock().unwrap();

    // Initialise params struct for current values in backend.
    // Any changes at UI not committed will be lost, where committed
    // implies parameters used for the currently generated fractal.
    let mut params = FractalParamsClear {
        rows: 0,
        cols: 0,
        centre_re: 0.0,
        centre_im: 0.0,
        pt_div: 0.0,
        max_its: 0,
    };

    // Assert parameters to current backend values.
    params.rows = fractal.rows;
    params.cols = fractal.cols;
    params.centre_re = fractal.mid_pt.re;
    params.centre_im = fractal.mid_pt.im;
    params.pt_div = fractal.pt_div;
    params.max_its = fractal.max_its;

    // Re-render the fractal image.
    fractal.render_image();

    // Report status and payload to front end.
    let render_time_ms:f64 = fractal.rendering_duration.as_millis() as f64 / 1000.0 as f64;
    let duration_str = format!("{:.3} sec", render_time_ms);

    // Ensure only the filename (not path) is sent to the frontend.
    let image_filename = std::path::Path::new(&fractal.image_filename)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let response_data = json!({
        "rendered": "True",
        "time": duration_str,
        "error": "Success",
        "params": params,
        "image": image_filename,
    });

    // Respond with status to display on UI.
    HttpResponse::Ok().json(response_data)
}

// User help endpoint.
async fn help(settings: web::Data<Settings>) -> impl Responder {
    // Help endpoint function.
    // Read the help file.
    let help_file_content = fs::read_to_string("./static/help.html")
        .expect("Unable to read help file");

    // Replace the version placeholder with the actual version number from settings.
    // Repeat as necessary for other setting information required in help.
    let help_content = help_file_content.replace("{{version}}", &settings.program_ver);

    HttpResponse::Ok().content_type("text/html").body(help_content)
}

// Application main.
// Sets up application folders and creates the fractal instance.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create folders if they don't already exist.
    fs::create_dir_all("./logs")?;
    fs::create_dir_all("./fractals")?;
    fs::create_dir_all("./palettes")?;

    // Logging configuration held in log4rs.yml .
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();
    // Do initial program version logging, mainly as a test.
    info!("Application started: {} v({})", settings.program_name, settings.program_ver);

    // Instantiate a fractals struct.
    // Call init method to initialise struct.
    let fractal = Arc::new(Mutex::new(Fractal::init()));

    // Check number of threads available for fractal computations.
    info!("Number of threads currently available for fractal processing: {}", rayon::current_num_threads());

    // Create and start web service.
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(fractal.clone()))
            .app_data(web::Data::new(settings.clone()))
            .service(fsx::Files::new("/fractals", "./fractals").show_files_listing())
            .service(intro)
            .service(generate)
            .service(recentre)
            .service(histogram)
            .service(palette)
            .service(render)
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .route("/help", web::get().to(help))
            .route("/fractals/{filename}", web::get().to(serve_image))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
