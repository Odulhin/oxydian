use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_extras::{Size, StripBuilder};
use egui_file_dialog::{FileDialog, FileDialogStorage};
// use egui_ltreeview::TreeView;
use std::path::PathBuf;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    label: String,
    text_buffer: String,
    #[serde(skip)]
    cache: CommonMarkCache,
    value: f32,
    #[serde(skip)]
    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            cache: CommonMarkCache::default(),
            text_buffer: String::new(),
            file_dialog: FileDialog::default(),
            picked_file: None,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.global_style_mut(|style| {
            // Show the url of a hyperlink on hover
            style.url_in_tooltip = true;
        });

        let mut app: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        if let Some(storage) = cc.storage {
            *app.file_dialog.storage_mut() =
                eframe::get_value::<FileDialogStorage>(storage, "file_dialog_storage")
                    .unwrap_or_default();
        }

        app
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
        eframe::set_value(
            storage,
            "file_dialog_storage",
            &self.file_dialog.storage_mut().clone(),
        );
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
                // egui::widgets::global_theme_preference_switch(ui);
            });
        });

        egui::Panel::left("side_panel").show_inside(ui, |ui| {
            if ui.button("Pick file").clicked() {
                // Open the file dialog to pick a file.
                self.file_dialog.pick_file();
            }

            ui.label(format!("Picked file: {:?}", self.picked_file));

            // Update the dialog
            self.file_dialog.update(ui);

            // Check if the user picked a file.
            if let Some(path) = self.file_dialog.take_picked() {
                self.picked_file = Some(path.to_path_buf());
            }
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading(format!("{:?}", self.picked_file));

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                StripBuilder::new(ui)
                    .sizes(Size::remainder(), 2)
                    .horizontal(|mut strip| {
                        strip.cell(|ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.text_buffer)
                                    .code_editor()
                                    .desired_width(f32::INFINITY),
                            );
                        });
                        strip.cell(|ui| {
                            CommonMarkViewer::new().max_image_width(Some(512)).show_mut(
                                ui,
                                &mut self.cache,
                                &mut self.text_buffer,
                            );
                        });
                    });
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
