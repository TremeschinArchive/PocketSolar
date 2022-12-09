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
use libm::*;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::api::WriteType;
use btleplug::platform::Manager;
use btleplug::api::bleuuid::uuid_from_u16;

use futures::executor::block_on;

// ----------------------------------------------------------------------------|

const BAUDRATE: u32 = 9600;

#[path = "ViyLine/Curve.rs"]
mod Curve;

#[path = "ViyLine/Bluetooth.rs"]
mod Bluetooth;

#[path = "ViyLine/Serial.rs"]
mod Serial;

#[path = "ViyLine/GUI.rs"]
mod GUI;

// ----------------------------------------------------------------------------|

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default)]
pub struct ViyLineApp {
    #[serde(skip)]
    ivCurve: Curve::Curve,

    // Plot options
    plotPoints: bool,
    plotCurve: bool,

    // Export Window
    showExportWindow: bool,
    exportNOfPoints: i64,
    outputCSV: String,

    // Bluetooth
    #[serde(skip)]
    hc06: Option<btleplug::platform::Peripheral>,
    #[serde(skip)]
    readCharacteristic:  Option<btleplug::api::Characteristic>,
    #[serde(skip)]
    writeCharacteristic: Option<btleplug::api::Characteristic>,

    // Serial
    #[serde(skip)]
    serialPort: Option<Box<dyn serialport::SerialPort>>,
    portName: String,
}

impl ViyLineApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> ViyLineApp {

        // Restore previous settings if any
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        return ViyLineApp {
            plotPoints: true,
            plotCurve: true,
            exportNOfPoints: 20,

            ..ViyLineApp::default()
        };
    }
}

// ----------------------------------------------------------------------------|

// Serial + Bluetooth. We can probably do better than this, returning Result
impl ViyLineApp {

    // Abstraction: Read 8 bits from the measure hardware
    fn picRead(&mut self) -> u8 {
        if self.serialPort.is_some() {
            self.openSerialPort(&self.portName.clone());
            return self.serialPortRead();
        }
        if self.hc06.is_some() {
            return self.bluetoothRead();
        };
        return 0;
    }

    // Abstraction: Write 8 bits from the measure hardware
    fn picWrite(&mut self, data: u8) {
        if self.serialPort.is_some() {
            self.openSerialPort(&self.portName.clone());
            return self.serialPortWrite(data);
        }
        if self.hc06.is_some() {
            return self.bluetoothWrite(data);
        };
    }
}

// ----------------------------------------------------------------------------|

async fn trueMain() {
    Protostar::setupLog();

    // Compile NATIVELY
    #[cfg(not(target_arch = "wasm32"))]
    eframe::run_native("ViyLine", eframe::NativeOptions::default(), Box::new(|cc| {
        let app = Box::new(ViyLineApp::new(cc));
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        return app;
    }));

    // Compile WASM
    #[cfg(target_arch = "wasm32")]
    {
        // Make sure panics are logged using `console.error`.
        console_error_panic_hook::set_once();

        // Redirect tracing to console.log and friends:
        tracing_wasm::set_as_global_default();

        eframe::start_web("ViyLine", eframe::WebOptions::default(), Box::new(|cc| {
            let app = Box::new(ViyLineApp::new(cc));
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            return app;
        })).expect("failed to start eframe");
    }
}

#[tokio::main(flavor="current_thread")]
#[cfg(not(target_arch="wasm32"))]
async fn main() {
    trueMain().await;
}

#[cfg(target_arch = "wasm32")]
fn main() {
    prokio::Runtime::default().spawn_pinned(move || async move {
        trueMain().await;
    });
}
