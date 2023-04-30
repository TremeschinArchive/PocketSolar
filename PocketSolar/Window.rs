use crate::*;

impl eframe::App for PocketSolarApp {

    // Save state on window shut down
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "PocketSolar", self);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }

    // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let solarPanelCurve = &mut self.solarPanelCurve.write().unwrap();

        // Top bar
        egui::TopBottomPanel::top("topPanel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("PocketSolar");

                // Serial port Combo Box connection
                ui.separator();
                ui.label("Serial Port");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{}", &mut solarPanelCurve.portName))
                    .show_ui(ui, |ui| {
                        let ports = serialport::available_ports();
                        ui.selectable_value(&mut solarPanelCurve.portName, str!("None"), str!("None"));
                        if ports.is_ok() {
                            for port in ports.unwrap().iter() {
                                let portName = port.port_name.clone();
                                ui.selectable_value(&mut solarPanelCurve.portName, portName.clone(), portName);
                            }
                        }
                    }
                ).response.on_hover_text("Select the serial port Arduino is connected to");

                // If we have measurements, show clear, export buttons
                if solarPanelCurve.points.len() > 0 {
                    if ui.button("Export").clicked() {
                        self.showExportWindow = !self.showExportWindow;
                    }
                }
            });

            // Repository, version
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("[v{}]", env!("CARGO_PKG_VERSION")));
                    ui.hyperlink_to("GitHub", "https://github.com/BrokenSource/PocketSolar");
                    egui::global_dark_light_mode_switch(ui);
                });
            });

            // Export points from curve window
            if self.showExportWindow {
                egui::Window::new("Export data window").show(ctx, |ui| {
                    ui.add(egui::Slider::new(&mut self.exportNOfPoints, 2..=100).text("Number of Points"));

                    ui.horizontal(|ui| {
                        ui.label("Export: ");
                        let analytic = ui.button("Analytic Curve").clicked();
                        let raw      = ui.button("Raw Measurements").clicked();

                        if raw || analytic {
                            self.outputCSV = str!("  i,     V,      I,        P\n");

                            if analytic {
                                // [A - Be^kx = 0] => [Be^kx = A] => [x = ln(A/B)/k]
                                let Voc = (solarPanelCurve.A/solarPanelCurve.B).ln() / solarPanelCurve.C;
                                let dV = Voc/(self.exportNOfPoints as f64 - 1.0);

                                // For every dv, calculate IV point
                                for i in 0..self.exportNOfPoints {
                                    let V = dV * (i as f64);
                                    let I = solarPanelCurve.currentAtVoltage(V);
                                    self.outputCSV.push_str(&format!("{:>3},{:>6.2},{:>7.4},{:>9.4}\n", i+1, V, I.abs(), V*I.abs()));
                                    if I < 0.0 {break;}
                                }
                            }

                            if raw {
                                for (i, point) in solarPanelCurve.points.iter().enumerate() {
                                    self.outputCSV.push_str(&format!("{:>3},{:>6.2},{:>7.4},{:>9.4}\n", i+1, point.voltage, point.current, point.voltage*point.current));
                                }
                            }
                        }
                    });

                    // Add text box
                    ui.add(egui::TextEdit::multiline(&mut self.outputCSV).font(egui::TextStyle::Monospace));
                });
            }
        });

        // Technical info
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {

            // Info and plot options
            ui.horizontal(|ui| {
                egui::warn_if_debug_build(ui);
                ui.label(format!("IV(v) = {:.4} - ({:.4e})exp({:.4}v)", solarPanelCurve.A, solarPanelCurve.B, solarPanelCurve.C)).on_hover_text("Estimated analytic IV curve equation");

                // Maximum power point
                ui.separator();
                ui.label(format!("MPP @ {:.2} V", solarPanelCurve.MPPVoltage)).on_hover_text("Maximum Power Point");

                ui.separator();
                ui.label("Plot:");
                ui.checkbox(&mut self.plotSolarCurve, "IV").on_hover_text("Plot the continuous IV curve");
                ui.checkbox(&mut self.plotPVcurve, "PV").on_hover_text("Plot the continuous PV curve");
                ui.checkbox(&mut self.plotPoints, "Raw").on_hover_text("Plot the raw measurements");
                ui.checkbox(&mut self.plotDuty, "Duty Cycle").on_hover_text("Plot the Duty Cycle of each point");

                // Amplification factors
                ui.separator();
                ui.label("Ki:");
                ui.add(egui::DragValue::new(&mut solarPanelCurve.Ki).speed(0.1).fixed_decimals(3)).on_hover_text("Current amplification factor relative to 5 V input on the Arduino to the real current");
                ui.label("Kv:");
                ui.add(egui::DragValue::new(&mut solarPanelCurve.Kv).speed(0.1).fixed_decimals(3)).on_hover_text("Voltage amplification factor relative to 5 V input on the Arduino to the real current");
                ui.separator();
            });
        });

        // Main plot
        egui::CentralPanel::default().show(ctx, |ui| {

            // Main Solar Panel Curves Plot
            Plot::new("solarPanelCurvesPlot")
                .show(ui, |plot_ui| {

                // Plot continuous IV curve
                if self.plotSolarCurve {
                    let curve = solarPanelCurve.clone();
                    plot_ui.line({
                        Line::new(PlotPoints::from_explicit_callback(
                            move |x| {
                                curve.currentAtVoltage(x)
                            }, .., 512,
                        ))
                        .width(5.0)
                    });
                }

                // Plot PV curve
                if self.plotPVcurve {
                    let curve = solarPanelCurve.clone();
                    plot_ui.line({
                        Line::new(PlotPoints::from_explicit_callback(
                            move |x| {
                                x*(7.0/260.0)*curve.currentAtVoltage(x)
                            }, .., 512,
                        ))
                        .width(5.0)
                    });
                }

                // Plot IV points on graph
                if self.plotPoints {
                    for point in &solarPanelCurve.points {
                        if point.current < 0.0 { continue; }
                        plot_ui.points(
                            Points::new([point.voltage, point.current])
                                .radius(5.0)
                                .color(Color32::from_rgb(0, 255, 0)),
                        );
                    }
                }

                // Plot dutyCycle on graph
                if self.plotDuty {
                    for point in &solarPanelCurve.points {
                        if point.current < 0.0 { continue; }
                        plot_ui.points(
                            Points::new([point.voltage, point.dutyCycle])
                                .radius(3.0)
                                .color(Color32::from_rgb(255, 255, 255)),
                        );
                    }
                }
            });
        });

        // Repaint always since we have a dynamic changing graph ;)
        // It's vsynced! Thanks egui
        ctx.request_repaint();
    }
}
