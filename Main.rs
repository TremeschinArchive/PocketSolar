#![cfg_attr(not(debug_assertions), windows_subsystem="windows")]
#![allow(non_snake_case)]
#![allow(unused_must_use)]
use Broken::*;

import!{PocketSolar}

use egui::plot::Line;
use egui::plot::Plot;
use egui::plot::PlotPoints;
use egui::plot::Points;
use egui::Color32;

#[derive(Parser, Debug)]
#[command(author=Broken::Constants::AUTHOR, about=Broken::Constants::About::PocketSolar, version)]
pub struct Args {
    #[arg(short, long, help = "Reset to default settings")]
    defaultSettings: bool,
}

BrokenStruct! {
    pub struct PocketSolarApp {
        #[serde(skip)]
        solarPanelCurve: Arc<RwLock<SolarCurve::SolarCurve>>,

        // Plot options
        #[default(true)]
        plotPoints: bool,
        #[default(true)]
        plotDuty: bool,
        #[default(true)]
        plotSolarCurve: bool,
        #[default(true)]
        plotPVcurve: bool,

        // Export Window
        showExportWindow: bool,
        #[default(20)]
        exportNOfPoints: i64,
        outputCSV: String,

        // Other configurations
        showConfigurationWindow: bool,
    }
}

impl PocketSolarApp {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> PocketSolarApp {

        // Restore previous settings if any
        let mut app = {
            if !args.defaultSettings {
                if let Some(storage) = cc.storage {
                    eframe::get_value(storage, "PocketSolar").unwrap_or_default()
                }
            }

            PocketSolarApp::default()
        };

        // Spin the SolarCurve thread
        app.solarPanelCurve = SolarCurve::SolarCurve::freewheelDefault();
        return app;
    }
}


fn main() {
    Broken::setupLog();
    let args = Args::parse();

    eframe::run_native("PocketSolar", eframe::NativeOptions::default(), Box::new(|cc| {
        let app = Box::new(PocketSolarApp::new(cc, args));
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        return app;
    }));
}
