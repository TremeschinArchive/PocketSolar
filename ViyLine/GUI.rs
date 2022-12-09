// | (c) 2022 Tremeschin, MIT License | ViyLine Project | //
use crate::*;

impl eframe::App for ViyLineApp {

    // Save state on window shut down
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Calculate IV curve
        let mut curve = self.ivCurve.clone();
        curve.calculateCoefficients();

        // Top bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            curve.addPoint(0.0, 0.0);

            ui.horizontal(|ui| {
                // Title
                ui.heading("ViyLine");
                ui.separator();

                // Dark mode / Light mode switch
                egui::global_dark_light_mode_switch(ui);

                // Buttons / Actions
                if ui.button("Measure").clicked() {

                    // Read an unsigned int 8 from serial port
                    fn readByte(app: &mut ViyLineApp) -> u8 {
                        app.picWrite(0x01);
                        return app.picRead();
                    }

                    // Times to measure
                    let times = match self.hc06 {
                        None => vec![5, 10, 15, 20, 30, 50],
                        Some(_) => vec![40],
                    };

                    for t in times {
                        info!(":: Measure with DeltaT = {t}ms");

                        // Clear and call new measuremen
                        self.picWrite(t);

                        // Syncronize PIC with Rust
                        while self.picRead() != 0xFF {
                            std::thread::sleep(std::time::Duration::from_millis(1));
                        }

                        std::thread::sleep(std::time::Duration::from_millis(10));

                        // Read measurements
                        for _ in 1..=20 {
                            let upperV = readByte(self) as f64;
                            let lowerV = readByte(self) as f64;
                            let upperI = readByte(self) as f64;
                            let lowerI = readByte(self) as f64;

                            let V = ((upperV*256.0 + lowerV)/1023.0) * 5.0;
                            let I = ((upperI*256.0 + lowerI)/1023.0) * 5.0;

                            info!("  [LW V: {upperV} {lowerV}] [LW I: {upperI} {lowerI}] [V {V:.4}] [I {I:.4}]");
                            self.ivCurve.addPoint(V, I);
                        }
                    }
                }


                // If we have measurements, show clear, export buttons
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

                            // Add text box
                            ui.add(egui::TextEdit::multiline(&mut self.outputCSV).font(egui::TextStyle::Monospace));
                        });
                    }
                }


                // Serial port Combo Box connection
                ui.separator();
                ui.label("Serial Port");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{}", &mut self.portName))
                    .show_ui(ui, |ui| {
                        let ports = serialport::available_ports();
                        if ports.is_ok() {
                            for port in ports.unwrap().iter() {
                                let portName = port.port_name.clone();
                                ui.selectable_value(&mut self.portName, portName.clone(), portName);
                            }
                        }
                    });


                // Bluetooth Connect Button
                ui.separator();
                if self.hc06.is_none() {
                    if ui.button("Connect Bluetooth").clicked() {
                        block_on(self.findBluetooth());
                    }
                } else {
                    ui.label("Bluetooth Connected");
                }

            });

            // Repository, version
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("[v{}]", env!("CARGO_PKG_VERSION")));
                    ui.hyperlink_to("GitHub", "https://github.com/BrokenSource/ViyLine");
                });
            });
        });

        // Technical info
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {

            // Info and plot options
            ui.horizontal(|ui| {
                egui::warn_if_debug_build(ui);
                ui.label(format!("Function:  i(v) = {:.2} - ({:.3e})exp({:.4}v)", curve.A, curve.B, curve.k));
                ui.checkbox(&mut self.plotPoints, "Plot Points");
                ui.checkbox(&mut self.plotCurve, "Plot Curve");
            });
        });

        // Main plot
        egui::CentralPanel::default().show(ctx, |ui| {

            // Main plot
            Plot::new("lines_demo").show(ui, |plot_ui| {

                // Plot continuous IV curve
                if self.plotCurve {
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
                }

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
