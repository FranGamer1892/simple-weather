/* window.rs
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

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use serde_json::Value;
use std::error::Error;

use std::sync::Mutex;
use std::sync::Arc;

use std::os::unix::process::CommandExt;
use std::process::Command;

static mut COORDINATES: Option<Arc<Mutex<Option<[f64; 2]>>>> = None;

pub fn set_coordinates(coordinates: Arc<Mutex<Option<[f64; 2]>>>) {
    // Use unsafe block to assign the shared state to the global variable
    unsafe {
        COORDINATES = Some(coordinates);
    }
}

pub fn get_coordinates() -> Option<[f64; 2]> {
    // Access the shared state and retrieve the coordinates
    unsafe {
        COORDINATES.as_ref().and_then(|coordinates| coordinates.lock().ok().unwrap().clone())
    }

}

struct WeatherInfo {
    location: String,
    country: String,
    temperature: f64,
    feels_like: f64,
    humidity: u64,
    pressure: f64,
    wind_direction: String,
    wind_speed: f64,
    gusts: f64,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnlug/SimpleWeather/window.ui")]
    pub struct SimpleWeatherWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub label2: TemplateChild<gtk::Label>,
        #[template_child]
        pub restart_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SimpleWeatherWindow {
        const NAME: &'static str = "SimpleWeatherWindow";
        type Type = super::SimpleWeatherWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SimpleWeatherWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();

            self.restart_button.connect_clicked(|_| {
                Command::new("/proc/self/exe").exec();
            });

            match get_coordinates() {
                Some(coords) => {
                    match fetch_weather(coords[0], coords[1]) {
                        Ok(weather_info) => {
                            let mut full_string = String::new();
                            full_string.push_str(&format!("Location: {}, {}\n", weather_info.location, weather_info.country));
                            full_string.push_str(&format!("Current temperature: {}°C\n", weather_info.temperature));
                            full_string.push_str(&format!("Feels like: {}°C\n", weather_info.feels_like));
                            full_string.push_str(&format!("Humidity: {}%\n", weather_info.humidity));
                            full_string.push_str(&format!("Air pressure: {} kPa\n", weather_info.pressure));
                            full_string.push_str(&format!("Wind: {} {} km/h\n", weather_info.wind_direction, weather_info.wind_speed));
                            full_string.push_str(&format!("Gusts: {} km/h", weather_info.gusts));

                            self.label2.set_text(full_string.as_str());
                        }
                        Err(error) => eprintln!("Failed to fetch weather info: {}", error),
                    }
                }
                None => eprintln!("Coordinates not set"),
            }
        }
    }
    impl WidgetImpl for SimpleWeatherWindow {}
    impl WindowImpl for SimpleWeatherWindow {}
    impl ApplicationWindowImpl for SimpleWeatherWindow {}
}

glib::wrapper! {
    pub struct SimpleWeatherWindow(ObjectSubclass<imp::SimpleWeatherWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl SimpleWeatherWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
    }
}

fn fetch_weather(latitude: f64, longitude: f64) -> Result<WeatherInfo, Box<dyn Error>> {
    let url = format!(
        "https://api.weatherapi.com/v1/current.json?key=YOUR_API_KEY_HERE&q={},{}",
        latitude, longitude
    );

    let response = reqwest::blocking::get(&url)?.text()?;
    let weather_data: Value = serde_json::from_str(&response)?;

    // Extract the relevant weather information from the JSON response
    let location = weather_data["location"]["name"].as_str().unwrap().to_owned();
    let country = weather_data["location"]["country"].as_str().unwrap().to_owned();
    let temperature = weather_data["current"]["temp_c"].as_f64().unwrap();
    let feels_like = weather_data["current"]["feelslike_c"].as_f64().unwrap();
    let humidity = weather_data["current"]["humidity"].as_u64().unwrap();
    let pressure = weather_data["current"]["pressure_mb"].as_f64().unwrap() / 10.0;
    let wind_direction = weather_data["current"]["wind_dir"].as_str().unwrap().to_owned();
    let wind_speed = weather_data["current"]["wind_kph"].as_f64().unwrap();
    let gusts = weather_data["current"]["gust_kph"].as_f64().unwrap();

    // Create and return the WeatherInfo struct
    let weather_info = WeatherInfo {
        location,
        country,
        temperature,
        feels_like,
        humidity,
        pressure,
        wind_direction,
        wind_speed,
        gusts,
    };

    Ok(weather_info)
}
