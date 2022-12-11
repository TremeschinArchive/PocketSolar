// | (c) 2022 Tremeschin, MIT License | ViyLine Project | //
use crate::*;

impl eframe::App for ViyLineApp {

    // Save state on window shut down
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "ViyLine", self);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }

    // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Top bar
        egui::TopBottomPanel::top("topPanel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Title
                ui.heading("ViyLine");
                ui.separator();

                // Configurations window
                if ui.button("🔧").clicked() {
                    self.showConfigurationWindow = !self.showConfigurationWindow;
                }

                // Buttons / Actions
                if ui.button("Measure").clicked() {
                    self.attemptOpenSerialPort();

                    // Read an unsigned int 8 from serial port
                    fn readByte(app: &mut ViyLineApp) -> Result<u8, ()> {
                        app.picWrite(0x01);
                        return app.picRead();
                    }

                    // Times to measure
                    let times = match self.hc06 {
                        None => vec![5, 10, 15, 20, 30, 50],
                        Some(_) => vec![40],
                    };

                    'timesLoop: for t in times {
                        info!(":: Measure with DeltaT = {t}ms");

                        // Clear and call new measuremen
                        self.picWrite(t);

                        // Syncronize PIC with Rust (wait for 0xFF else break on error)
                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(1));
                            let value = self.picRead();
                            match value {
                                Ok(value) => {if value == 0xFF {break;}},
                                Err(()) => {
                                    error!("Error on reading from PIC, either no SerialPort or no Bluetooth");
                                    break 'timesLoop;
                                },
                            };
                        }

                        // Read measurements
                        for p in 1..=20 {
                            info!("Reading 4 bytes for Point #{p}");
                            let upperV = readByte(self).unwrap() as f64;
                            let lowerV = readByte(self).unwrap() as f64;
                            let upperI = readByte(self).unwrap() as f64;
                            let lowerI = readByte(self).unwrap() as f64;

                            // (0-100%) * Scaler (out of 5V)
                            let V = ((upperV*256.0 + lowerV)/1023.0) * self.Ki;
                            let I = ((upperI*256.0 + lowerI)/1023.0) * self.Kv;

                            info!("  [LW V: {upperV} {lowerV}] [LW I: {upperI} {lowerI}] [V {V:.4}] [I {I:.4}]");
                            self.solarPanelCurve.addPoint(V, I);
                        }
                    }

                    self.calculateRegression();
                }

                // If we have measurements, show clear, export buttons
                if self.solarPanelCurve.points.len() > 0 {
                    if ui.button("Clear").clicked() {
                        self.solarPanelCurve.clear();
                    }

                    if ui.button("Export").clicked() {
                        self.showExportWindow = !self.showExportWindow;
                    }
                }

                // Serial port Combo Box connection
                ui.separator();
                ui.label("Serial Port");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{}", &mut self.portName))
                    .show_ui(ui, |ui| {
                        let ports = serialport::available_ports();
                        ui.selectable_value(&mut self.portName, String::from("None"), String::from("None"));
                        if ports.is_ok() {
                            for port in ports.unwrap().iter() {
                                let portName = port.port_name.clone();
                                ui.selectable_value(&mut self.portName, portName.clone(), portName);
                            }
                        }
                    }
                );

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
                    egui::global_dark_light_mode_switch(ui);
                });
            });

            // Export points from curve window
            if self.showExportWindow {
                egui::Window::new("Export data window").show(ctx, |ui| {
                    ui.add(egui::Slider::new(&mut self.exportNOfPoints, 2..=100).text("Number of Points"));

                    // // (re)Build the output CSV
                    if ui.button("Export CSV").clicked() {

                        // Final export deserves more computation
                        for _ in 1..20 {self.calculateRegression(); }

                        // Start with keys
                        self.outputCSV = String::from("index,     V,      I\n");

                        // V open circuit
                        // [A - Be^kx = 0] => [Be^kx = A] => [x = ln(A/B)/k]
                        let Voc = (self.solarPanelCurve.A/self.solarPanelCurve.B).ln() / self.solarPanelCurve.C;
                        let dV = Voc/(self.exportNOfPoints as f64 - 1.0);

                        // For every dv
                        for i in 0..self.exportNOfPoints {
                            // Calculate next IV point
                            let V = dV * (i as f64);
                            let I = self.solarPanelCurve.interpolatedValueAt(V);
                            self.outputCSV.push_str(&format!("{:>5},{:>6.2},{:>7.4}\n", i, V, I.abs()));
                            if I < 0.0 {break;}
                        }
                    }

                    // Add text box
                    ui.add(egui::TextEdit::multiline(&mut self.outputCSV).font(egui::TextStyle::Monospace));
                });
            }

            // Configuration window
            if self.showConfigurationWindow {
                egui::Window::new("Configurations").show(ctx, |ui| {
                    egui::Grid::new("configurationWindowGrid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Bluetooth Name:");
                            ui.add(egui::TextEdit::singleline(&mut self.viylineHardwareBluetoothDeviceName).hint_text("HC-06 Configured Name"));
                            ui.end_row();

                            // Regression related
                            ui.label("Regression steps:");
                            ui.add(egui::DragValue::new(&mut self.regressionSteps).speed(1).clamp_range(1..=100));
                            ui.end_row();

                            // Checkboxes
                            ui.checkbox(&mut self.recalculateRegressionOnCoefficientChanges, "Recalculate Regression");
                            ui.checkbox(&mut self.plotInteractive, "Interactive Plot");
                            ui.end_row();

                            if ui.button("Add synthetic points").clicked() {
                                self.solarPanelCurve.addPoint( 0.0, 10.0);
                                self.solarPanelCurve.addPoint(40.0, 9.34);
                                self.solarPanelCurve.addPoint(50.0,  0.0);
                                self.solarPanelCurve.calculateCoefficients(self.regressionSteps);
                            }

                            if ui.button("Recalculate regression").clicked() {
                                self.calculateRegression();
                            }

                            ui.end_row();
                        });
                });
            }
        });

        // Technical info
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {

            // Info and plot options
            ui.horizontal(|ui| {
                egui::warn_if_debug_build(ui);
                ui.label(format!("IV(v) = {:.2} - ({:.3e})exp({:.4}v)", self.solarPanelCurve.A, self.solarPanelCurve.B, self.solarPanelCurve.C));

                ui.separator();
                ui.label("Plot curve:");
                ui.checkbox(&mut self.plotIVcurve, "IV");
                ui.checkbox(&mut self.plotPVcurve, "PV");

                // Amplification factors
                ui.separator();
                ui.label("Ki:").on_hover_text("Current amplification factor relative to 5 V input on the microcontroller");
                ui.add(egui::DragValue::new(&mut self.Ki).speed(0.1).fixed_decimals(3));
                ui.label("Kv:").on_hover_text("Voltage amplification factor relative to 5 V input on the microcontroller");
                ui.add(egui::DragValue::new(&mut self.Kv).speed(0.1).fixed_decimals(3));

                ui.separator();
                ui.checkbox(&mut self.plotPoints, "Plot Points");
            });
        });

        // Main plot
        egui::CentralPanel::default().show(ctx, |ui| {

            // Main Solar Panel Curves Plot
            Plot::new("solarPanelCurvesPlot")
                .allow_zoom(self.plotInteractive)
                .allow_scroll(self.plotInteractive)
                .allow_boxed_zoom(self.plotInteractive)
                .allow_drag(self.plotInteractive)
                // .auto_bounds_x()
                .show(ui, |plot_ui| {

                // Plot continuous IV curve
                if self.plotIVcurve {
                    let curve = self.solarPanelCurve.clone();
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

                // Plot PV curve
                if self.plotPVcurve {
                    let curve = self.solarPanelCurve.clone();
                    plot_ui.line({
                        Line::new(PlotPoints::from_explicit_callback(
                            move |x| {
                                if x < 0.0 {return 0.0;}
                                let I = curve.interpolatedValueAt(x);
                                if I < 0.0 {return 0.0;}
                                return I*x * (7.0/260.0);
                            }, .., 512,
                        ))
                        .width(5.0)
                    });
                }

                // Plot points on graph
                if self.plotPoints {
                    for point in &self.solarPanelCurve.points {
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
