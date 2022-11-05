// | (c) 2022 Tremeschin, AGPLv3-only License | PhasorFlow Project | //
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[derive(Default)]
pub struct ViyLineApp {
    label: String,
    value: f32,
}

impl ViyLineApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> ViyLineApp {
        return ViyLineApp {
            label: "Hi".to_string(),
            value: 2.0,
            ..ViyLineApp::default()
        };
    }
}

impl eframe::App for ViyLineApp {
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("ViyLine");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));

            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    egui::warn_if_debug_build(ui);
                    //         ui.spacing_mut().item_spacing.x = 0.0;
                    //         ui.label("powered by ");
                    //         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    //         ui.label(" and ");
                    //         ui.hyperlink_to(
                    //             "eframe",
                    //             "https://github.com/emilk/egui/tree/master/crates/eframe",
                    //         );
                    //         ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
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
