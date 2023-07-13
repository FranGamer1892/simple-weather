/* main.rs
 *
 * Copyright 2023 fran
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod application;
mod config;
mod window;

use self::application::SimpleWeatherApplication;
use self::window::SimpleWeatherWindow;

use ipgeolocate::{Locator, Service};
use std::error::Error;
use tokio::runtime::Runtime;

use std::sync::Mutex;
use std::sync::Arc;

use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::gio;
use gtk::prelude::*;

fn main() {

    let coordinates: Arc<Mutex<Option<[f64; 2]>>> = Arc::new(Mutex::new(None));

    // Create a tokio runtime
    let rt = Runtime::new().unwrap();

    // Run the main function in the tokio runtime
    rt.block_on(async {
        // Call the async function
        let result = get_coordinates().await;

        // Process the result
        match result {
            Ok((latitude, longitude)) => {
                // Acquire a lock on the coordinates mutex
                let mut coordinates_lock = coordinates.lock().unwrap();
                // Modify the value inside the mutex
                *coordinates_lock = Some([latitude, longitude]);
            }
            Err(error) => {
                // Handle the error
                eprintln!("Error: {}", error);
            }
        }
    });

    window::set_coordinates(coordinates.clone());

    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Load resources
    let resources = gio::Resource::load(PKGDATADIR.to_owned() + "/simple-weather.gresource")
        .expect("Could not load resources");
    gio::resources_register(&resources);

    // Create a new GtkApplication. The application manages our main loop,
    // application windows, integration with the window manager/compositor, and
    // desktop features such as file opening and single-instance applications.
    let app = SimpleWeatherApplication::new("org.gnlug.SimpleWeather", &gio::ApplicationFlags::empty());

    // Run the application. This function will block until the application
    // exits. Upon return, we have our exit code to return to the shell. (This
    // is the code you see when you do `echo $?` after running a command in a
    // terminal.
    std::process::exit(app.run());
}

async fn get_coordinates() -> Result<(f64, f64), Box<dyn Error>> {
    let service = Service::IpApi;

    if let Some(ip) = public_ip::addr().await {
        let ip_string = ip.to_string();
        let ip_str: &str = ip_string.as_str();
        match Locator::get(ip_str, service).await {
            Ok(ip_str) => {
                println!("{},{}", ip_str.latitude, ip_str.longitude);
                let latitude: f64 = ip_str.latitude.parse().expect("Failed to parse the string as f64");
                let longitude: f64 = ip_str.longitude.parse().expect("Failed to parse the string as f64");
                return Ok((latitude, longitude)); // Add return statement to return successful result
            }
            Err(error) => return Err(error.into()), // Return Err variant with the error value
        }
    } else {
        eprintln!("couldn't get an IP address");
    }

    Err("Failed to retrieve coordinates".into()) // Return Err variant for failure
}
