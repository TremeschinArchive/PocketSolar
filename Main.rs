#![cfg_attr(not(debug_assertions), windows_subsystem="windows")]
#![allow(non_snake_case)]
#![allow(unused_must_use)]
use Broken::*;

import!{PocketSolar}

#[derive(Parser, Debug)]
#[command(author=Broken::Constants::AUTHOR, about=Broken::Constants::About::PocketSolar, version)]
pub struct Args {
    #[arg(short, long, help = "Reset to default settings")]
    defaultSettings: bool,
}

// ----------------------------------------------------------------------------|

use egui::plot::Line;
use egui::plot::Plot;
use egui::plot::PlotPoints;
use egui::plot::Points;
use egui::Color32;

const BAUDRATE: u32 = 9600;

// ----------------------------------------------------------------------------|

BrokenStruct! {
    pub struct PocketSolarApp {
        solarPanelCurve: SolarCurve::SolarCurve,

        // Current, voltage amplification factor
        #[default(1.0)]
        Ki: f64,
        #[default(1.0)]
        Kv: f64,

        // Plot options
        #[default(true)]
        plotPoints: bool,
        #[default(true)]
        plotSolarCurve: bool,
        #[default(true)]
        plotPVcurve: bool,

        // Export Window
        showExportWindow: bool,
        #[default(20)]
        exportNOfPoints: i64,
        outputCSV: String,

        // Serial
        #[serde(skip)]
        #[derivative(Debug="ignore")]
        serialPort: Option<Arc<dyn serialport::SerialPort>>,
        #[default(str!("None"))]
        portName: String,

        // Other configurations
        showConfigurationWindow: bool,

        // Regression
        #[default(100)]
        regressionSteps: i64,
        recalculateRegressionOnCoefficientChanges: bool,
    }
}

impl PocketSolarApp {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> PocketSolarApp {

        // Restore previous settings if any
        if !args.defaultSettings {
            if let Some(storage) = cc.storage {
                return eframe::get_value(storage, "PocketSolar").unwrap_or_default();
            }
        }

        PocketSolarApp::default()
    }
}

// ----------------------------------------------------------------------------|

fn main() {
    Broken::setupLog();
    let args = Args::parse();

    eframe::run_native("PocketSolar", eframe::NativeOptions::default(), Box::new(|cc| {
        let app = Box::new(PocketSolarApp::new(cc, args));
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        return app;
    }));
}
