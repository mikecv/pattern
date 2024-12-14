// Fractals application.

use log::info;
use log4rs;
use actix_files as fsx;
use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use lazy_static::lazy_static;
use num_complex::Complex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
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

#[get("/")]
async fn intro() -> impl Responder {
    info!("Invoking UI intro endpoint.");

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
#[derive(Deserialize)]
struct FractalParams {
    value1: u32,
    value2: u32,
    value3: f64,
    value4: f64,
    value5: f64,
    value6: u32,
}

#[post("/generate")]
async fn generate(fractal_params: web::Json<FractalParams>, fractal: web::Data<Arc<Mutex<Fractal>>>,) -> impl Responder {
    info!("Invoking fractal generation endpoint.");

    // Get application settings in scope.
    let _settings: Settings = SETTINGS.lock().unwrap().clone();

    // Get access to steg instance.
    let mut fractal = fractal.lock().unwrap();

    // Access parameters.
    // Check if any parameters are set to 0 and if so set to default.
    // For parameters 3 and 4 that are real and imaginary parts of the midpoint,
    // they are allowed to be 0, so just use the entered values.
    let params = fractal_params.into_inner();
    if params.value1 != 0 as u32 {
        fractal.rows = params.value1;
    }
    if params.value2 != 0 as u32 {
        fractal.cols = params.value2;
    }
    fractal.mid_pt = Complex::new(params.value3, params.value4);
    if params.value5 != 0 as f64 {
        fractal.pt_div = params.value5;
    }
    if params.value6 != 0 as u32 {
        fractal.max_its = params.value6;
    }

    // Initialise the fractal.
    fractal.init_fractal_image();

    // Generate the fractal.
    // and report status and payload to front end.
    match fractal.generate_fractal(){
        Ok(_) => {
            // Fractal generation successful, respond with status.
            let mut response_data = HashMap::new();
            response_data.insert("generation", "True".to_string());
            let test_time_ms:f64 = fractal.generate_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", test_time_ms);
            response_data.insert("time", duration_str);
            response_data.insert("error", "Success".to_string());

             // Respond with status to display on UI.
             HttpResponse::Ok().json(response_data)
        }
        Err(e) => {
            // Fractal generation failed, respond with error.
            let mut response_data = HashMap::new();
            response_data.insert("generation", "False".to_string());
            let test_time_ms:f64 = fractal.generate_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", test_time_ms);
            response_data.insert("time", duration_str);
            response_data.insert("error", e.to_string());
 
             // Respond with status to display on UI.
             HttpResponse::InternalServerError().json(response_data)
        }
    }
}
    
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create folders if they don't already exist.
    fs::create_dir_all("./logs")?;
    fs::create_dir_all("./fractals")?;

    // Logging configuration held in log4rs.yml .
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();
    // Do initial program version logging, mainly as a test.
    info!("Application started: {} v({})", settings.program_name, settings.program_ver);

    // Instantiate a fractals struct.
    // Call init method to initialise struct.
    let fractal = Arc::new(Mutex::new(Fractal::init()));

    // Create and start web service.
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(fractal.clone()))
            .app_data(web::Data::new(settings.clone()))
            .service(fsx::Files::new("/fractals", "./fractals").show_files_listing())
            .service(intro)
            .service(generate)
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .route("/help", web::get().to(help))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
