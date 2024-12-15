// Fractals application.

use log::info;
use log4rs;
use actix_files as fsx;
use actix_files::NamedFile;
use actix_web::{get, post, web, App, HttpRequest, HttpServer, HttpResponse, Responder};
use lazy_static::lazy_static;
use num_complex::Complex;
use serde::{Deserialize, Serialize};
use serde_json::json;
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

// File server.
async fn serve_image(_req: HttpRequest, path: web::Path<String>) -> actix_web::Result<NamedFile> {
    let file_path = format!("./fractals/{}", path.into_inner());
    Ok(NamedFile::open(file_path)?)
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
#[derive(Deserialize, Serialize, Debug, Clone)]
struct FractalParams {
    value1: Option<u32>,
    value2: Option<u32>,
    value3: Option<f64>,
    value4: Option<f64>,
    value5: Option<f64>,
    value6: Option<u32>,
}

#[post("/generate")]
async fn generate(fractal_params: web::Json<FractalParams>, fractal: web::Data<Arc<Mutex<Fractal>>>,) -> impl Responder {
    info!("Invoking fractal generation endpoint.");

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();

    // Get access to steg instance.
    let mut fractal = fractal.lock().unwrap();

    // Access parameters.
    // Check if any parameters are set to none type,
    // if so, set to default setting.
    let mut params = fractal_params.into_inner();

    // Parameter 1.
    params.value1 = Some(params.value1.unwrap_or(settings.init_rows));
    fractal.rows = params.value1.unwrap();

    // Parameter 2.
    params.value2 = Some(params.value2.unwrap_or(settings.init_cols));
    fractal.cols = params.value2.unwrap();

    // Parameter 3 & 4
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

    // Initialise fractal.
    fractal.init_fractal_image();

    // Initialise colour palette.
    let _ = fractal.init_col_pallete();

    // Generate the fractal.
    // and report status and payload to front end.
    match fractal.generate_fractal(){
        Ok(_) => {
            let test_time_ms:f64 = fractal.generate_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", test_time_ms);

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

            let test_time_ms:f64 = fractal.generate_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", test_time_ms);

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

    // Create and start web service.
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(fractal.clone()))
            .app_data(web::Data::new(settings.clone()))
            .service(fsx::Files::new("/fractals", "./fractals").show_files_listing())
            .service(intro)
            .service(generate) // The `generate` handler for `/generate` endpoint
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .route("/help", web::get().to(help))
            .route("/fractals/{filename}", web::get().to(serve_image))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
