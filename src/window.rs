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

use titlecase::titlecase;

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
    weather_description: String,
    temperature: f64,
    humidity: u64,
    pressure: f64,
    wind_cardinal_direction: String,
    wind_speed: f64,
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
                            full_string.push_str(&format!("Description: {}\n", weather_info.weather_description));
                            full_string.push_str(&format!("Current temperature: {}°C\n", weather_info.temperature));
                            full_string.push_str(&format!("Humidity: {}%\n", weather_info.humidity));
                            full_string.push_str(&format!("Air pressure: {} kPa\n", weather_info.pressure));
                            full_string.push_str(&format!("Wind: {} {} km/h", weather_info.wind_cardinal_direction, weather_info.wind_speed));

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

fn deg_to_cardinal(wind_direction: f64) -> String {
    let directions = ["N", "NE", "E", "SE", "S", "SW", "W", "NW", "N"];

    let index = ((wind_direction + 22.5) / 45.0).floor() as usize % 8;
    directions[index].to_string()
}

fn fetch_weather(latitude: f64, longitude: f64) -> Result<WeatherInfo, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=metric&appid={API_key}",
        latitude, longitude
    );

    let response = reqwest::blocking::get(&url)?.text()?;
    let weather_data: Value = serde_json::from_str(&response)?;

    // Extract the relevant weather information from the JSON response
    let location = weather_data["name"].as_str().unwrap().to_owned();
    let country = weather_data["sys"]["country"].as_str().unwrap().to_owned();
    let weather_description = match weather_data["weather"][0]["description"].as_str() {
        Some(desc) => {
            let capitalized_desc = titlecase(desc);
            capitalized_desc
        }
        None => {
            eprintln!("Warning: Weather description not available");
            String::new()
        }
    };
    let temperature = (weather_data["main"]["temp"].as_f64().unwrap() * 10.0).round() / 10.0;
    let humidity = weather_data["main"]["humidity"].as_u64().unwrap();
    let pressure = weather_data["main"]["pressure"].as_f64().unwrap() / 10.0;
    let wind_direction_deg = weather_data["wind"]["deg"].as_f64().unwrap();
    let wind_cardinal_direction = deg_to_cardinal(wind_direction_deg);
    let wind_speed = (weather_data["wind"]["speed"].as_f64().unwrap() * 3.6 * 10.0).round() / 10.0;

    // Create and return the WeatherInfo struct
    let weather_info = WeatherInfo {
        location,
        country,
        temperature,
        weather_description,
        humidity,
        pressure,
        wind_cardinal_direction,
        wind_speed,
    };

    Ok(weather_info)
}
