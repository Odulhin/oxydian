use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_extras::{Size, StripBuilder};
use egui_file_dialog::{FileDialog, FileDialogStorage};
use egui_ltreeview::{TreeView, TreeViewState};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct TreeEntry {
    id: u32,
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<TreeEntry>,
}

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
    #[serde(skip)]
    tree_state: TreeViewState<u32>,
    #[serde(skip)]
    cached_root: Option<PathBuf>,
    cached_entries: Option<Vec<TreeEntry>>,
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
            tree_state: TreeViewState::default(),
            cached_root: None,
            cached_entries: None,
        }
    }
}

impl App {
    /// Builds a cached tree structure from a directory path.
    fn build_cached_tree(&mut self, path: &Path) -> Vec<TreeEntry> {
        let mut id_counter = 0u32;
        Self::build_tree_recursive(path, &mut id_counter)
    }

    /// Renders the tree view from cached entries.
    fn render_tree_from_cache(
        builder: &mut egui_ltreeview::TreeViewBuilder<'_, u32>,
        entries: &[TreeEntry],
    ) {
        for entry in entries {
            if entry.is_dir {
                builder.dir(entry.id, &entry.name);
                Self::render_tree_from_cache(builder, &entry.children);
                builder.close_dir();
            } else {
                builder.leaf(entry.id, &entry.name);
            }
        }
    }

    /// Sets all directory entries to be closed in the tree state.
    fn close_all_directories_in_state(&mut self, entries: &[TreeEntry]) {
        for entry in entries {
            if entry.is_dir {
                self.tree_state.set_openness(entry.id, false);
                self.close_all_directories_in_state(&entry.children);
            }
        }
    }

    /// Recursively builds tree entries from filesystem.
    fn build_tree_recursive(path: &Path, id_counter: &mut u32) -> Vec<TreeEntry> {
        let mut entries = Vec::new();

        if let Ok(read_dir) = fs::read_dir(path) {
            let mut dir_entries: Vec<_> = read_dir
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect();

            // Sort: directories first, then alphabetically
            dir_entries.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                if a_is_dir != b_is_dir {
                    b_is_dir.cmp(&a_is_dir)
                } else {
                    let a_name = a.file_name().unwrap_or_default().to_string_lossy();
                    let b_name = b.file_name().unwrap_or_default().to_string_lossy();
                    a_name.cmp(&b_name)
                }
            });

            for entry_path in dir_entries {
                let name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let is_dir = entry_path.is_dir();
                let current_id = *id_counter;
                *id_counter += 1;

                if is_dir {
                    let children = Self::build_tree_recursive(&entry_path, id_counter);
                    entries.push(TreeEntry {
                        id: current_id,
                        name,
                        path: entry_path,
                        is_dir: true,
                        children,
                    });
                } else {
                    entries.push(TreeEntry {
                        id: current_id,
                        name,
                        path: entry_path,
                        is_dir: false,
                        children: Vec::new(),
                    });
                }
            }
        }

        entries
    }

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
                        if ui.button("Pick folder").clicked() {
                            // Open the file dialog to pick a folder.
                            self.file_dialog.pick_directory();
                        }
                        // Update the dialog
                        self.file_dialog.update(ui);
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
                // egui::widgets::global_theme_preference_switch(ui);
            });
        });

        egui::Panel::left("side_panel").show_inside(ui, |ui| {
            // if ui.button("Pick folder").clicked() {
            //     // Open the file dialog to pick a folder.
            //     self.file_dialog.pick_directory();
            // }
            // ui.separator();
            // ui.label(format!("Picked folder: {:?}", self.picked_file));

            // // Update the dialog
            // self.file_dialog.update(ui);

            // Rebuild cached entries if we have a picked file but no cached entries
            // (happens on first load with persisted picked_file)
            if self.picked_file.is_some() && self.cached_entries.is_none() {
                if let Some(root_path) = &self.picked_file {
                    let root_to_scan = if root_path.is_dir() {
                        root_path.clone()
                    } else if let Some(parent) = root_path.parent() {
                        parent.to_path_buf()
                    } else {
                        root_path.clone()
                    };
                    let entries = self.build_cached_tree(&root_to_scan);
                    self.cached_entries = Some(entries.clone());
                    self.tree_state = TreeViewState::default();
                    self.close_all_directories_in_state(&entries);
                }
            }

            // Check if the user picked a file.
            if let Some(path) = self.file_dialog.take_picked() {
                let new_path = path.to_path_buf();
                // Only rebuild if path actually changed
                if self.picked_file.as_ref() != Some(&new_path) {
                    self.picked_file = Some(new_path.clone());
                    self.cached_root = Some(new_path.clone());

                    // Build cached tree from the directory (or parent if file selected)
                    let root_to_scan = if new_path.is_dir() {
                        new_path.clone()
                    } else if let Some(parent) = new_path.parent() {
                        parent.to_path_buf()
                    } else {
                        new_path.clone()
                    };

                    let entries = self.build_cached_tree(&root_to_scan);
                    self.cached_entries = Some(entries.clone());
                    self.tree_state = TreeViewState::default();
                    self.close_all_directories_in_state(&entries);
                }
            }

            egui::ScrollArea::both().show(ui, |ui| {
                TreeView::new(egui::Id::new("tree view")).show_state(
                    ui,
                    &mut self.tree_state,
                    |builder| {
                        if let Some(entries) = &self.cached_entries {
                            Self::render_tree_from_cache(builder, entries);
                        }
                    },
                );
            });
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
