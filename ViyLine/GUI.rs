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
                ui.heading("ViyLine");
                ui.separator();

                // Configurations window
                if ui.button("🔧").clicked() {
                    self.showConfigurationWindow = !self.showConfigurationWindow;
                }

                // Buttons / Actions
                if ui.button("Measure").clicked() {
                    self.openSerialPort(&self.portName.clone());

                    if self.serialPort.is_some() {

                        // Read an unsigned int 8 from serial port
                        fn readByte(app: &mut ViyLineApp) -> u8 {
                            app.serialPortWrite(0x01);
                            return app.serialPortRead();
                        }

                        // Times to measure
                        let times = vec![5, 15, 50, 100, 200];

                        for t in times {
                            info!(":: Measure with DeltaT = {t}ms");

                            // Clear and call new measuremen
                            self.serialPortWrite(t);

                            // Syncronize PIC with Rust (wait for 0xFF else break on error)
                            while self.serialPortRead() != 0xFF {
                                std::thread::sleep(std::time::Duration::from_millis(1));
                            }

                            // Read measurements
                            for _ in 1..=20 {
                                let upperV = readByte(self) as f64;
                                let lowerV = readByte(self) as f64;
                                let upperI = readByte(self) as f64;
                                let lowerI = readByte(self) as f64;

                                // (0-100%) * Scaler (out of 5V)
                                let V = ( (upperV*256.0 + lowerV)/1023.0) * 5.0 * self.Kv;
                                let I = ( (upperI*256.0 + lowerI)/1023.0) * 5.0 * self.Ki;

                                info!("  [LW V: {upperV} {lowerV}] [LW I: {upperI} {lowerI}] [V {V:.4}] [I {I:.4}]");
                                self.solarPanelCurve.addPoint(V, I);
                            }
                        }

                        self.updateSolarPanelCurve();
                    }
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
                        ui.selectable_value(&mut self.portName, str!("None"), str!("None"));
                        if ports.is_ok() {
                            for port in ports.unwrap().iter() {
                                let portName = port.port_name.clone();
                                ui.selectable_value(&mut self.portName, portName.clone(), portName);
                            }
                        }
                    }
                );
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

                    // Prepare variables for export; Final export deserves more computation
                    fn commonExport(app: &mut ViyLineApp) {
                        for _ in 1..3 {app.updateSolarPanelCurve()}
                        app.outputCSV = str!("  i,     V,      I,        P\n");
                    }

                    ui.horizontal(|ui| {
                        ui.label("Export: ");

                        if ui.button("Analytic Curve").clicked() {
                            commonExport(self);

                            // [A - Be^kx = 0] => [Be^kx = A] => [x = ln(A/B)/k]
                            let Voc = (self.solarPanelCurve.A/self.solarPanelCurve.B).ln() / self.solarPanelCurve.C;
                            let dV = Voc/(self.exportNOfPoints as f64 - 1.0);

                            // For every dv, calculate IV point
                            for i in 0..self.exportNOfPoints {
                                let V = dV * (i as f64);
                                let I = self.solarPanelCurve.currentAtVoltage(V);
                                self.outputCSV.push_str(&format!("{:>3},{:>6.2},{:>7.4},{:>9.4}\n", i+1, V, I.abs(), V*I.abs()));
                                if I < 0.0 {break;}
                            }
                        }

                        if ui.button("Raw Measurements").clicked() {
                            commonExport(self);
                            for (i, point) in self.solarPanelCurve.points.iter().enumerate() {
                                self.outputCSV.push_str(&format!("{:>3},{:>6.2},{:>7.4},{:>9.4}\n", i+1, point.v, point.i, point.v*point.i));
                            }
                        }
                    });

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
                            // Regression related
                            ui.label("Regression steps:");
                            ui.add(egui::DragValue::new(&mut self.regressionSteps).speed(1).clamp_range(1..=100));
                            ui.end_row();

                            ui.checkbox(&mut self.recalculateRegressionOnCoefficientChanges, "Clean Regression")
                                .on_hover_text("Reset regression coefficients on every calculation");
                            if ui.button("Recalculate regression").clicked() { self.updateSolarPanelCurve(); }
                            ui.end_row();

                            if ui.button("Add synthetic points").clicked() {
                                self.solarPanelCurve.addPoint( 0.0, 3.0);
                                self.solarPanelCurve.addPoint(40.0, 2.84);
                                self.solarPanelCurve.addPoint(50.0,  0.0);
                                self.solarPanelCurve.calculateCoefficients(self.regressionSteps);
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
                ui.label(format!("IV(v) = {:.4} - ({:.4e})exp({:.4}v)", self.solarPanelCurve.A, self.solarPanelCurve.B, self.solarPanelCurve.C));

                // Maximum power point
                ui.separator();
                ui.label(format!("MPP @ {:.2} V", self.solarPanelCurve.MPPVoltage));

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
                .show(ui, |plot_ui| {

                // Plot continuous IV curve
                if self.plotIVcurve {
                    let curve = self.solarPanelCurve.clone();
                    plot_ui.line({
                        Line::new(PlotPoints::from_explicit_callback(
                            move |x| {
                                return curve.currentAtVoltage(x);
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
                                return x*(7.0/260.0)*curve.currentAtVoltage(x);
                            }, .., 512,
                        ))
                        .width(5.0)
                    });
                }

                // Plot points on graph
                if self.plotPoints {
                    for point in &self.solarPanelCurve.points {
                        if point.i < 0.0 { continue; }
                        plot_ui.points(
                            Points::new([point.v, point.i])
                                .radius(2.0)
                                .color(Color32::from_rgb(0, 255, 0)),
                        );
                    }
                }
            });
        });
    }
}
