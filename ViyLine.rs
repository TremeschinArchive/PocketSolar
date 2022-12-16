// | (c) 2022 Tremeschin, MIT License | ViyLine Project | //
// #![cfg_attr(not(debug_assertions), windows_subsystem="windows")]
#![allow(non_snake_case)]
#![allow(unused_must_use)]
use Protostar::*;

use egui::plot::Line;
use egui::plot::Plot;
use egui::plot::PlotPoints;
use egui::plot::Points;
use egui::Color32;

// ----------------------------------------------------------------------------|

const BAUDRATE: u32 = 9600;

#[path = "ViyLine/Curve.rs"]
mod Curve;

#[path = "ViyLine/Serial.rs"]
mod Serial;

#[path = "ViyLine/GUI.rs"]
mod GUI;

// ----------------------------------------------------------------------------|

const ABOUT: &str = "
ViyLine, a Solar Panel IV Curve Tracker

(c) Tremeschin, AGPLv3-only License.";

// CLI Arguments
#[derive(Parser, Debug)]
#[command(author="Tremeschin", version, about=ABOUT, long_about=None)]
pub struct Args {
    // Reset settings on boot
    #[arg(short, long, help = "Reset to default settings")]
    defaultSettings: bool,
}

// ----------------------------------------------------------------------------|

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default)]
pub struct ViyLineApp {
    // #[serde(skip)]
    solarPanelCurve: Curve::Curve,

    // Hardware configuration

    // Current, voltage amplification factor
    Ki: f64,
    Kv: f64,
    offset: f64,

    // Plot options
    plotPoints: bool,
    plotIVcurve: bool,
    plotPVcurve: bool,
    plotInteractive: bool,

    // Export Window
    showExportWindow: bool,
    exportNOfPoints: i64,
    outputCSV: String,

    // Serial
    #[serde(skip)]
    serialPort: Option<Box<dyn serialport::SerialPort>>,
    portName: String,

    // Other configurations
    showConfigurationWindow: bool,

    // Regression
    regressionSteps: i64,
    recalculateRegressionOnCoefficientChanges: bool,
}

impl ViyLineApp {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> ViyLineApp {

        // Restore previous settings if any
        if let Some(storage) = cc.storage {
            if !args.defaultSettings {
                return eframe::get_value(storage, "ViyLine").unwrap_or_default();
            }
        }

        // Default configuration
        return ViyLineApp {

            // Current, voltage amplification factor
            Ki: 1.0,
            Kv: 10.0,
            offset: 2.5,

            // Plot options
            plotPoints: true,
            plotIVcurve: true,
            plotPVcurve: true,
            plotInteractive: false,

            // Export
            exportNOfPoints: 20,

            // Serial
            portName: String::from("None"),

            // Regression
            regressionSteps: 100,
            recalculateRegressionOnCoefficientChanges: false,

            ..ViyLineApp::default()
        };
    }
}

// ----------------------------------------------------------------------------|

async fn trueMain() {
    Protostar::setupLog();
    let args = Args::parse();

    // Compile NATIVELY
    #[cfg(not(target_arch = "wasm32"))]
    eframe::run_native("ViyLine", eframe::NativeOptions::default(), Box::new(|cc| {
        let app = Box::new(ViyLineApp::new(cc, args));
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        return app;
    }));
}

#[tokio::main(flavor="current_thread")]
async fn main() {
    trueMain().await;
}
