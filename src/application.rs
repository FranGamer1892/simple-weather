/* application.rs
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

use crate::config::VERSION;
use crate::SimpleWeatherWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SimpleWeatherApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for SimpleWeatherApplication {
        const NAME: &'static str = "SimpleWeatherApplication";
        type Type = super::SimpleWeatherApplication;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for SimpleWeatherApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.instance();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
        }
    }

    impl ApplicationImpl for SimpleWeatherApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.instance();
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = SimpleWeatherWindow::new(&*application);
                window.set_resizable(false);
                window.set_title(Some("Simple Weather"));
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for SimpleWeatherApplication {}
    }

glib::wrapper! {
    pub struct SimpleWeatherApplication(ObjectSubclass<imp::SimpleWeatherApplication>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SimpleWeatherApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::new(&[("application-id", &application_id), ("flags", flags)])
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]).unwrap();
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = gtk::AboutDialog::builder()
            .transient_for(&window)
            .modal(true)
            .program_name("Simple Weather")
            .logo_icon_name("org.gnlug.SimpleWeather")
            .version(VERSION)
            .license_type(gtk::License::Gpl30)
            .authors(vec!["Francisco Zadikian".into()])
            .copyright("© 2023 Francisco Zadikian")
            .build();

        about.present();
    }
}
