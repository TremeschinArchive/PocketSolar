// | (c) 2022 Tremeschin, MIT License | ViyLine Project | //
// #![cfg_attr(not(debug_assertions), windows_subsystem="windows")]
#![allow(non_snake_case)]

use egui::plot::Line;
use egui::plot::Plot;
use egui::plot::PlotPoints;
use egui::plot::Points;
use egui::Color32;
use libm::*;
use rand::Rng;
use rand_pcg::Pcg32;
use rand::SeedableRng;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::api::WriteType;
use btleplug::platform::Manager;
use btleplug::api::bleuuid::uuid_from_u16;

// ----------------------------------------------------------------------------|

#[derive(Default, Clone)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Default, Clone)]
struct Curve {
    points: Vec<Point>,

    // Curve parameters
    k: f64,
    A: f64,
    B: f64
}

impl Curve {
    // Returns the coefficients
    fn calculateCoefficients(&mut self) {

        // If we even have some points
        if self.points.len() > 0 {

            // Initial guess of B value
            if self.B == 0.0 {self.B = 0.5;}

            // This is one of the hardest part, find the perfect initial value I(0)
            let maxY = self.minMaxY().unwrap().1;

            // Repeat until we get a nice estimate of B
            for _ in 1..50 {

                // Update A coefficient based on last iteration values
                self.A = maxY + self.B;

                // X, Y points for linear regression
                let x = Vec::from_iter(self.points.iter().map( |point|  point.x                ));
                let y = Vec::from_iter(self.points.iter().map( |point| (self.A*(1.0) - point.y).ln() ));

                // Linear regression
                let sumX:   f64 = x.iter().sum();
                let sumY:   f64 = y.iter().sum();
                let sumXY:  f64 = x.iter().zip(y).map(|(a, b)| a*b).sum();
                let sumXSq: f64 = x.iter().map(|a| a*a).sum();
                let n = self.points.len() as f64;

                // y = ax + b
                let a = (n*sumXY - sumX*sumY)/(n*sumXSq - sumX.powf(2.0));
                let b = (sumY - a*sumX)/n;

                // On the linearized iv curve, for y = A - Be^kx, we have ln(y) = -kx + ln(B)
                self.k = a;
                self.B = exp(b);
            }
        }
    }

    // Calculate a generic point X
    fn interpolatedValueAt(&self, x: f64) -> f64 {
        return self.A - self.B*exp(self.k*x);
    }

    // Minimum and maximum value of the curve
    fn minMaxY(&self) -> Option<(f64, f64)> {
        if self.points.len() == 0 {return None;}
        let mut yValues = Vec::from_iter(self.points.iter().map(|point| point.y));
        yValues.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let minY = yValues.first().unwrap();
        let maxY = yValues.last().unwrap();
        return Some((*minY, *maxY));
    }

    // Empty the curve
    fn clear(&mut self) {
        self.points = Vec::new();
    }

    fn addPoint(&mut self, x: f64, y: f64) {
        self.points.push(Point { x: x, y: y });
    }
}

// ----------------------------------------------------------------------------|

#[derive(Default)]
pub struct ViyLineApp {
    ivCurve: Curve,

    time: f64,

    // Plot
    plotPoints: bool,

    // Export Window
    showExportWindow: bool,
    exportNOfPoints: i64,
    outputCSV: String,

    // Temporary variables
    A: f64,
    B: f64,
    N: i64,
    k: f64,
    errorPCT: f64,
    errorRange: f64,

    // Bluetooth
    hc06: Option<btleplug::platform::Peripheral>,
    readCharacteristic:  Option<btleplug::api::Characteristic>,
    writeCharacteristic: Option<btleplug::api::Characteristic>,
    bluetoothDevices: Vec<String>,
}


impl ViyLineApp {
    pub async fn new() -> ViyLineApp {
        return ViyLineApp {
            plotPoints: true,

            exportNOfPoints: 20,

            A: 10.0,
            B: 0.01,
            k: 0.30,
            N: 79,

            errorPCT: 20.0,
            errorRange: 0.03,

            ..ViyLineApp::default()
        };
    }

    fn bluetoothWrite(&self, data: &Vec<u8>) {
        block_on(self.hc06.as_ref().unwrap().write(&self.writeCharacteristic.as_ref().unwrap(), data, WriteType::WithoutResponse)).unwrap();
        // block_on(async_std::task::sleep(std::time::Duration::from_millis(50)));
    }

    fn bluetoothRead(&self) -> Vec<u8> {
        let data = block_on(self.hc06.as_ref().unwrap().read(&self.readCharacteristic.as_ref().unwrap())).unwrap();
        // block_on(async_std::task::sleep(std::time::Duration::from_millis(50)));
        return data;
    }

    async fn findBluetooth(&mut self) {

        let manager = Manager::new().await.unwrap();
        let adapter_list = manager.adapters().await.unwrap();

        // Add number adapters found
        self.bluetoothDevices.push(format!("Number of Adapters found: [{}]", adapter_list.len()));

        // For all found Bluetooth adapters
        for adapter in adapter_list.iter() {
            self.bluetoothDevices.push(
                format!("Adapter: [{}]", adapter.adapter_info().await.unwrap())
            );

            // Scan for peripherals
            adapter.start_scan(ScanFilter::default()).await.unwrap();
            async_std::task::sleep(std::time::Duration::from_millis(2000)).await;

            // FIXME: Wasm unreachable
            let peripherals = adapter.peripherals().await.unwrap_or(Vec::new());

            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await.expect("Can't get properties");
                let local_name = properties.unwrap().local_name.unwrap_or(String::from("Unknown Name"));

                // Only connect to HC-06
                if local_name != "HC-06" {continue;}

                // Connect if not paired
                if !peripheral.is_connected().await.unwrap() {
                    if let Err(err) = peripheral.connect().await {
                        self.bluetoothDevices.push(format!(" - ERROR: {}", err));
                        continue;
                    }
                }

                // Show info on name
                self.bluetoothDevices.push(format!(" - {}", local_name));

                // Discover services and characteristics
                peripheral.discover_services().await.unwrap();
                let characteristics = Some(peripheral.characteristics().clone());
                self.writeCharacteristic = Some(characteristics.as_ref().unwrap().iter().find(|c| c.uuid == uuid_from_u16(0xFFE2)).unwrap().clone());
                self.readCharacteristic  = Some(characteristics.as_ref().unwrap().iter().find(|c| c.uuid == uuid_from_u16(0xFFE1)).unwrap().clone());

                // Assign bluetooth module variables
                self.hc06 = Some(peripheral.clone());
            }

        }
    }
}

impl eframe::App for ViyLineApp {

    // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.time += 1.0/60.0;

        // Calculate IV curve
        let mut curve = self.ivCurve.clone();
        curve.calculateCoefficients();

        // Top bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {

            ui.horizontal(|ui| {
                // Title
                ui.heading("ViyLine");
                ui.separator();

                // Dark mode / Light mode switch
                egui::global_dark_light_mode_switch(ui);

                if self.hc06.is_none() {
                    if ui.button("Connect").clicked() {
                        block_on(self.findBluetooth());
                    }
                } else {
                    // Buttons / Actions
                    if ui.button("Measure").clicked() {
                        for i in 0..10  {
                            println!("");

                            // Write data
                            self.bluetoothWrite(&vec![(i) as u8, (i+1) as u8, (i+2) as u8, (i+3) as u8]);

                            // Read data
                            let data = self.bluetoothRead();
                            println!("Buffer: {:?}", data);
                        }
                    }
                }

                if curve.points.len() > 0 {
                    if ui.button("Clear").clicked() {
                        self.ivCurve.clear();
                    }

                    if ui.button("Export").clicked() {
                        self.showExportWindow = !self.showExportWindow;
                    }

                    if self.showExportWindow {
                        egui::Window::new("Export data window").show(ctx, |ui| {
                            ui.add(egui::Slider::new(&mut self.exportNOfPoints, 2..=100).text("Number of Points"));

                            // // (re)Build the output CSV
                            if ui.button("Export CSV").clicked() {
                                self.outputCSV = String::from("index,     V,      I\n");

                                // V open circuit
                                // [A - Be^kx = 0] => [Be^kx = A] => [x = ln(A/B)/k]
                                let Voc = (curve.A/curve.B).ln() / curve.k;
                                let dV = Voc/(self.exportNOfPoints as f64 - 1.0);

                                // For every dv
                                for i in 0..self.exportNOfPoints {
                                    // Calculate next IV point
                                    let V = dV * (i as f64);
                                    let I = curve.interpolatedValueAt(V);
                                    self.outputCSV.push_str(&format!("{:>5},{:>6.2},{:>7.4}\n", i, V, I.abs()));
                                    if I < 0.0 {break;}
                                }
                            }

                            // // Add text box
                            ui.add(
                                egui::TextEdit::multiline(&mut self.outputCSV)
                                    .font(egui::TextStyle::Monospace)

                            );
                        });
                    }
                }
            });

            // Repository, version
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("[v{}]", env!("CARGO_PKG_VERSION")));
                    ui.hyperlink_to("GitHub", "https://github.com/BrokenSource/ViyLine");
                    // egui::warn_if_debug_build(ui);
                });
            });
        });


        // Technical info
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label(format!("Function:  i(v) = {:.2} - ({:.3e})exp({:.4}v)", curve.A, curve.B, curve.k));
            ui.style_mut().spacing.slider_width = 260.0;

            // Temporary unknown variables sliders
            if true {
                ui.add(egui::Slider::new(&mut self.A, 0.0..=10.0).text("Unknown A"));
                ui.add(egui::Slider::new(&mut self.B, 0.0..= 2.0).text("Unknown B"));
                ui.add(egui::Slider::new(&mut self.k, 0.0..= 1.0).text("Unknown k"));
                ui.separator();
                ui.add(egui::Slider::new(&mut self.N, 0..=500).text("Points"));
                ui.separator();
                ui.add(egui::Slider::new(&mut self.errorPCT, 0.0..=100.0).text("Error %"));
                ui.add(egui::Slider::new(&mut self.errorRange, 0.0..=0.99).text("Error ABS"));
            }
        });


        // Main plot
        egui::CentralPanel::default().show(ctx, |ui| {

            // Temporary random points
            if true {
                let mut rng = Pcg32::seed_from_u64(42);
                self.ivCurve.clear();

                for _ in 0..self.N {

                    // Random X, precise Y
                    let x = rng.gen_range(0.0..30.0);
                    let mut y = self.A - self.B*exp(self.k*x);

                    // Insert dirty Y sometimes
                    if rng.gen_range(0.0..100.0) > (100.0 - self.errorPCT) {
                        if self.errorRange != 0.0 {
                            y *= rng.gen_range((1.0 - self.errorRange)..1.0);
                        }
                    }

                    self.ivCurve.addPoint(x, y);
                }
            }

            // Main plot
            Plot::new("lines_demo").show(ui, |plot_ui| {

                // Plot continuous IV curve
                plot_ui.line({
                    Line::new(PlotPoints::from_explicit_callback(
                        move |x| {
                            if x < 0.0 {return 0.0;}
                            let I = curve.interpolatedValueAt(x);
                            if I < 0.0 {return 0.0;}
                            return I;
                        }, .., 512,
                    ))
                    .width(5.0)
                });

                // Plot points on graph
                if self.plotPoints {
                    for point in &self.ivCurve.points {
                        if point.y < 0.0 { continue; }
                        plot_ui.points(
                            Points::new([point.x, point.y])
                                .radius(2.0)
                                .color(Color32::from_rgb(0, 255, 0)),
                        );
                    }
                }
            });
        });

        egui::Window::new("Bluetooth").show(ctx, |ui| {
            for name in &self.bluetoothDevices {
                ui.label(name);
            }
        });

    }
}

use futures::executor::block_on;


async fn trueMain() {
    let app = Box::new(ViyLineApp::new().await);

    // Compile NATIVELY
    #[cfg(not(target_arch = "wasm32"))]
    eframe::run_native("ViyLine", eframe::NativeOptions::default(), Box::new(|cc| {
        cc.egui_ctx.set_visuals(egui::Visuals::dark()); return app;
    }));

    // Compile WASM
    #[cfg(target_arch = "wasm32")]
    {
        // Make sure panics are logged using `console.error`.
        console_error_panic_hook::set_once();

        // Redirect tracing to console.log and friends:
        tracing_wasm::set_as_global_default();

        eframe::start_web("ViyLine", eframe::WebOptions::default(), Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark()); return app;
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
