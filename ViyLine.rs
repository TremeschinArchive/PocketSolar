// | (c) 2022 Tremeschin, MIT License | ViyLine Project | //
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
    numberInterpolationPoints: i64,
    ivCurve: Curve,

    time: f64,

    // Plot
    plotPoints: bool,

    // Export Window
    showExportWindow: bool,
    exportdV: f64,
    outputCSV: String,

    // Temporary variables
    A: f64,
    B: f64,
    N: i64,
    k: f64,
    errorPCT: f64,
    errorRange: f64,
}

impl ViyLineApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> ViyLineApp {
        return ViyLineApp {
            numberInterpolationPoints: 10,

            plotPoints: true,

            exportdV: 0.5,

            A: 10.0,
            B: 0.01,
            k: 0.30,
            N: 79,

            errorPCT: 20.0,
            errorRange: 0.03,

            ..ViyLineApp::default()
        };
    }
}

impl eframe::App for ViyLineApp {
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
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

                // Buttons / Actions
                if ui.button("Measure").clicked() {

                }

                if ui.button("Clear").clicked() {
                    self.ivCurve.clear();
                }

                if ui.button("Export").clicked() {
                    self.showExportWindow = !self.showExportWindow;
                }

                if self.showExportWindow {
                    egui::Window::new("Export data window").show(ctx, |ui| {
                        ui.add(egui::Slider::new(&mut self.exportdV, 0.1..=5.0).text("Voltage Precision"));

                        // // (re)Build the output CSV
                        if ui.button("Export CSV").clicked() {
                            self.outputCSV = String::from("V,I\n");

                            // For every dv
                            for i in 0.. {
                                // Calculate next IV point
                                let V = self.exportdV * (i as f64);
                                let I = curve.interpolatedValueAt(V);
                                self.outputCSV.push_str(&format!("{:.2},{:.4}\n", V, I));
                                if I < 0.0 {break;}
                            }
                        }

                        // // Add text box
                        ui.add(
                            egui::TextEdit::multiline(&mut self.outputCSV)
                        );
                    });
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

            // Temporary
            ui.add(egui::Slider::new(&mut self.A, 0.0..=10.0).text("Unknown A"));
            ui.add(egui::Slider::new(&mut self.B, 0.0..= 2.0).text("Unknown B"));
            ui.add(egui::Slider::new(&mut self.k, 0.0..= 1.0).text("Unknown k"));
            ui.separator();
            ui.add(egui::Slider::new(&mut self.N, 0..=500).text("Points"));
            ui.separator();
            ui.add(egui::Slider::new(&mut self.errorPCT, 0.0..=100.0).text("Error %"));
            ui.add(egui::Slider::new(&mut self.errorRange, 0.0..=0.99).text("Error ABS"));
        });




        // egui::SidePanel::left("side_panel").show(ctx, |ui| {
        // Temporary point adder
        //     ui.separator();
        //     ui.add(egui::Slider::new(&mut self._randomX, 0.0..=10.0).text("X: "));
        //     ui.add(egui::Slider::new(&mut self._randomY, 0.0..=10.0).text("Y: "));
        //     if ui.button("Add point").clicked() {
        //         self.ivCurve.addPoint(self._randomX, self._randomY);
        //     }
        //     ui.separator();


        egui::CentralPanel::default().show(ctx, |ui| {
            // // Interpolation precision
            // ui.add(
            //     egui::Slider::new(&mut self.numberInterpolationPoints, 0..=100)
            //         .text("Interpolation points"),
            // );

            // Plot interpolation
            let plot = Plot::new("lines_demo");
            self.ivCurve.addPoint(-1.0, 0.0);

            {
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

            plot.show(ui, |plot_ui| {

                // Plot continuous IV curve
                plot_ui.line({
                    Line::new(PlotPoints::from_explicit_callback(
                        move |x| curve.interpolatedValueAt(x), .., 512,
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



    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "ViyLine",
        native_options,
        Box::new(|cc| Box::new(ViyLineApp::new(cc))),
    );
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "mainCanvas",
        web_options,
        Box::new(|cc| Box::new(ViyLineApp::new(cc))),
    )
    .expect("failed to start eframe");
}














// let mut y = 0.0;

// // Interpolation core
// for (n, point) in (&self.points).iter().enumerate() {
//     y += point.y * exp(-((x - point.x) / 0.1).powf(2.0));
// }

// return y / 1.7724538509055159;



            // // Make a linspace
            // for i in 0..npoints {
            //     // Theoretical X, start with zero y-value
            //     let wantX = minX + (maxX - minX) * ((i as f64) / (npoints as f64));

            //     // Append point
            //     curve.points.push(
            //         Point {
            //             x: wantX,
            //             y: A - B*exp(k*wantX),
            //         }
            //     );
            // }