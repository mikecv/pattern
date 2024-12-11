// Fractals application.

use log::info;
use log4rs;
use actix_files as fsx;
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use lazy_static::lazy_static;
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

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();

    // Assign parameter defaults so that they can be passed to UI.
    let default_rows = settings.init_rows;
    let default_cols = settings.init_cols;
    let default_centre_re = settings.init_mid_pt_re;
    let default_centre_im = settings.init_mid_pt_im;
    let default_division = settings.init_pt_div;
    let default_max_its = settings.init_max_its;

    // In the UI replace the parameter tags with the default values.
    let html = include_str!("../static/index.html")
        .replace("{{ default_rows }}", &default_rows.to_string())
        .replace("{{ default_cols }}", &default_cols.to_string())
        .replace("{{ default_centre_re }}", &default_centre_re.to_string())
        .replace("{{ default_centre_im }}", &default_centre_im.to_string())
        .replace("{{ default_division }}", &default_division.to_string())
        .replace("{{ default_max_its }}", &default_max_its.to_string());

        HttpResponse::Ok().content_type("text/html").body(html)
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
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .route("/help", web::get().to(help))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
