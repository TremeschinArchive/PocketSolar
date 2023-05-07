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
                ui.label(format!("MPP @ {:.02} V, {:.02} W", solarPanelCurve.MPPVoltage, solarPanelCurve.MPPPower())).on_hover_text("Maximum Power Point");

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
            let plotHeight = (ui.available_height()/3.0) * 0.99;
            let plotAspectRatio = ((ui.available_height()/3.0)/ui.available_width()) as f64;

            // "Overhshoot" bounds
            let extraBoundX = 1.02;
            let extraBoundY = 1.0 + extraBoundX*plotAspectRatio;

            // For Duty Cycle plot
            let maxVoltage = solarPanelCurve.minMaxX().unwrap_or([0.0, 0.0])[1];

            Plot::new("ivCurve").height(plotHeight).allow_drag(false).allow_scroll(false).allow_zoom(false).show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max([0.0, 0.0], [
                    solarPanelCurve.minMaxX().unwrap_or([0.0, 0.0])[1]*extraBoundX,
                    solarPanelCurve.minMaxY().unwrap_or([5.0, 5.0])[1]*extraBoundY
                ]));

                // Plot analytic IV curve
                let curve = solarPanelCurve.clone();
                plot_ui.line({
                    Line::new(PlotPoints::from_explicit_callback(
                        move |x| {
                            curve.currentAtVoltage(x)
                        }, .., 512,
                    ))
                    .width(5.0)
                });

                // Plot raw measurements
                for point in &solarPanelCurve.points {
                    if point.current < 0.0 { continue; }
                    plot_ui.points(
                        Points::new([point.voltage, point.current])
                            .radius(2.0)
                            .color(Color32::from_rgb(0, 100, 255)),
                    );
                }
            });

            Plot::new("pvCurve").height(plotHeight).allow_drag(false).allow_scroll(false).allow_zoom(false).show_background(false).show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max([0.0, 0.0], [
                    maxVoltage*extraBoundX,
                    solarPanelCurve.MPPPower()*extraBoundY]
                ));

                // Plot analytic PV curve
                let curve = solarPanelCurve.clone();
                plot_ui.line({
                    Line::new(PlotPoints::from_explicit_callback(
                        move |x| {
                            x*curve.currentAtVoltage(x)
                        }, .., 512,
                    ))
                    .width(5.0)
                    .color(Color32::from_rgb(0, 0, 255))
                });
            });

            Plot::new("dutyCycle").height(plotHeight).allow_drag(false).allow_scroll(false).allow_zoom(false).show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max([0.0, 0.0], [
                    maxVoltage*extraBoundX,
                    extraBoundY]
                ));

                // Plot dutyCycle on graph
                for point in &solarPanelCurve.points {
                    if point.current < 0.0 { continue; }
                    plot_ui.points(
                        Points::new([point.voltage, point.dutyCycle])
                            .radius(3.0)
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                }
            });

        });

        // Repaint always since we have a dynamic changing graph ;)
        // It's vsynced! Thanks egui
        ctx.request_repaint();
    }
}
