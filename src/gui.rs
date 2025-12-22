use eframe::egui;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use arboard::Clipboard;
use rfd::FileDialog;
use crate::backends::get_backend;
use crate::process;

pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 600.0])
            .with_resizable(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "Steam HDR Launcher",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

#[derive(PartialEq)]
enum Tab {
    Steam,
    EpicWrapper,
}

struct MyApp {
    // Shared
    monitors: Vec<String>,
    selected_monitor: Option<String>,
    wait_process: String,
    status_msg: String,

    // State
    current_tab: Tab,
    
    // Steam Tab
    generated_cmd: String,

    // Epic Tab
    target_exe: String,
    selected_game_path: Option<String>,

    // Process Picker
    show_process_picker: bool,
    process_list: Vec<String>,
    process_filter: String,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let backend = get_backend();
        let monitors = backend.get_monitors().unwrap_or_else(|_| vec!["Error scanning".to_string()]);
        
        let selected = if !monitors.is_empty() {
            Some(monitors[0].clone())
        } else {
            None
        };

        Self {
            monitors,
            selected_monitor: selected,
            wait_process: String::new(),
            status_msg: String::new(),
            current_tab: Tab::Steam,
            generated_cmd: String::new(),
            target_exe: "Game_Original.exe".to_string(),
            selected_game_path: None,
            show_process_picker: false,
            process_list: Vec::new(),
            process_filter: String::new(),
        }
    }

    // --- UTILS ---
    #[allow(dead_code)] // Intentionally unused for now, logic inline in update
    fn toggle_test_hdr(&mut self) {
        // Logic moved inline to update loop to avoid borrowing issues or simplify
    }
    
    fn refresh_process_list(&mut self) {
        self.process_list = process::get_running_processes_list();
    }

    fn open_windows_settings(&self) {
        let _ = Command::new("cmd")
            .args(&["/C", "start", "ms-settings:display-advanced"])
            .spawn();
    }

    // --- STEAM LOGIC ---
    fn generate_steam_command(&mut self) {
        if let Some(monitor) = &self.selected_monitor {
            if let Ok(exe_path) = env::current_exe() {
                let exe_str = exe_path.to_string_lossy();
                let wait_arg = if !self.wait_process.trim().is_empty() {
                    format!("--wait \"{}\" ", self.wait_process.trim()) 
                } else {
                    String::new()
                };
                self.generated_cmd = format!("\"{}\" \"{}\" {}-- %command%", exe_str, monitor, wait_arg);
                self.status_msg = "Steam command generated!".to_string();
            }
        }
    }

    fn copy_to_clipboard(&mut self) {
        if self.generated_cmd.is_empty() { return; }
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&self.generated_cmd) {
                    self.status_msg = format!("Clipboard error: {}", e);
                } else {
                    self.status_msg = "Copied to clipboard!".to_string();
                }
            },
            Err(e) => self.status_msg = format!("Clipboard init error: {}", e),
        }
    }

    // --- EPIC / WRAPPER LOGIC ---
    fn browse_game_executable(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Executable", &["exe"])
            .set_title("Select Game Executable")
            .pick_file() 
        {
            let path_str = path.to_string_lossy().to_string();
            self.selected_game_path = Some(path_str);

            if let Some(file_stem) = path.file_stem() {
                 let stem_str = file_stem.to_string_lossy();
                 self.target_exe = format!("{}_Original.exe", stem_str);
            }
        }
    }
    
    fn check_is_installed(&self) -> bool {
        if let Some(path_str) = &self.selected_game_path {
            let path = Path::new(path_str);
            if let Some(parent) = path.parent() {
                let config_path = parent.join("hdr_config.ini");
                let target_path = parent.join(&self.target_exe);
                return config_path.exists() && target_path.exists();
            }
        }
        false
    }

    fn install_wrapper(&mut self) {
        let monitor = match &self.selected_monitor {
            Some(m) => m,
            None => {
                self.status_msg = "Error: No monitor selected.".to_string();
                return;
            }
        };

        let game_path_str = match &self.selected_game_path {
            Some(p) => p,
            None => {
                self.status_msg = "Error: No game executable selected.".to_string();
                return;
            }
        };

        let game_path = Path::new(game_path_str);
        let game_dir = match game_path.parent() {
            Some(p) => p,
            None => {
                self.status_msg = "Error: Invalid game path.".to_string();
                return;
            }
        };

        let target_exe_name = self.target_exe.trim();
        if target_exe_name.is_empty() {
             self.status_msg = "Error: Target executable name cannot be empty.".to_string();
             return;
        }

        let new_target_path = game_dir.join(target_exe_name);

        if !new_target_path.exists() {
             if let Err(e) = fs::rename(game_path, &new_target_path) {
                self.status_msg = format!("Error renaming game exe: {}", e);
                return;
             }
        } else {
             if !game_path.exists() {
             } else {
                 self.status_msg = format!("Error: Target '{}' already exists. Rename manually or delete it.", target_exe_name);
                 return;
             }
        }

        if let Ok(current_exe) = env::current_exe() {
            if let Err(e) = fs::copy(&current_exe, game_path) {
                self.status_msg = format!("Error copying launcher: {}", e);
                let _ = fs::rename(&new_target_path, game_path);
                return;
            }
        } else {
             self.status_msg = "Error: Could not locate self.".to_string();
             return;
        }

        let wait = self.wait_process.trim();
        let content = format!(
            "monitor={}\ntarget={}\nwait={}",
            monitor, target_exe_name, wait
        );
        let ini_path = game_dir.join("hdr_config.ini");

        match fs::write(ini_path, content) {
            Ok(_) => self.status_msg = "Success! Wrapper installed.".to_string(),
            Err(e) => self.status_msg = format!("Error writing config: {}", e),
        }
    }
    
    fn uninstall_wrapper(&mut self) {
         let game_path_str = match &self.selected_game_path {
            Some(p) => p,
            None => return,
        };

        let game_path = Path::new(game_path_str);
        let game_dir = match game_path.parent() {
            Some(p) => p,
            None => return,
        };

        let ini_path = game_dir.join("hdr_config.ini");
        if ini_path.exists() {
            let _ = fs::remove_file(ini_path);
        }

        if game_path.exists() {
             if let Err(e) = fs::remove_file(game_path) {
                 self.status_msg = format!("Error removing wrapper: {}", e);
                 return;
             }
        }

        let target_path = game_dir.join(&self.target_exe);
        if target_path.exists() {
            if let Err(e) = fs::rename(&target_path, game_path) {
                 self.status_msg = format!("Error restoring original game: {}", e);
            } else {
                 self.status_msg = "Success! Wrapper uninstalled.".to_string();
            }
        } else {
            self.status_msg = "Warning: Original game file not found to restore.".to_string();
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // PROCESS PICKER WINDOW
        // Fix: Use a local variable to control open state to avoid double borrow of self
        let mut show_picker = self.show_process_picker;
        
        if show_picker {
             // We need to clone the data needed inside the closure so we don't borrow self
             let process_list = self.process_list.clone();
             let mut filter = self.process_filter.clone();
             let mut selected_process: Option<String> = None;

             egui::Window::new("Select Running Process")
                .open(&mut show_picker)
                .collapsible(false)
                .resizable(false)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Filter:");
                        ui.text_edit_singleline(&mut filter);
                    });
                    ui.separator();
                    
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        let filter_lower = filter.to_lowercase();
                        for proc in &process_list {
                             if filter_lower.is_empty() || proc.to_lowercase().contains(&filter_lower) {
                                  if ui.button(proc).clicked() {
                                      selected_process = Some(proc.clone());
                                  }
                             }
                        }
                    });
                });
            
            // Sync back changes
            self.process_filter = filter;
            
            if let Some(proc) = selected_process {
                self.wait_process = proc;
                self.show_process_picker = false;
            } else {
                self.show_process_picker = show_picker;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("HDR Launcher Setup");
            
            // SYSTEM SETTINGS LINK
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                 if ui.link("⚙ Windows HDR Settings").clicked() {
                     self.open_windows_settings();
                 }
            });
            ui.add_space(5.0);

            // --- TABS ---
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Steam, "Steam Command");
                ui.selectable_value(&mut self.current_tab, Tab::EpicWrapper, "Epic/Standalone");
            });
            ui.separator();
            ui.add_space(10.0);

            // --- COMMON: MONITOR SELECT ---
            ui.label("1. Select HDR Monitor:");
            ui.horizontal(|ui| {
                let selected_text = self.selected_monitor.as_deref().unwrap_or("No monitors found");
                egui::ComboBox::from_id_salt("monitor_combo")
                    .selected_text(selected_text)
                    .width(250.0)
                    .show_ui(ui, |ui| {
                        for monitor in &self.monitors {
                            ui.selectable_value(&mut self.selected_monitor, Some(monitor.clone()), monitor);
                        }
                    });
                
                if ui.button("Refresh").clicked() {
                    let backend = get_backend();
                    if let Ok(new_list) = backend.get_monitors() {
                        self.monitors = new_list;
                        if !self.monitors.is_empty() { self.selected_monitor = Some(self.monitors[0].clone()); }
                    }
                }
            });
            
            // TEST HDR ROW
            ui.horizontal(|ui| {
                ui.label("Test HDR:");
                if ui.button("ON").clicked() {
                     if let Some(m) = &self.selected_monitor {
                         let _ = get_backend().toggle_hdr(m, true);
                     }
                }
                if ui.button("OFF").clicked() {
                     if let Some(m) = &self.selected_monitor {
                         let _ = get_backend().toggle_hdr(m, false);
                     }
                }
            });

            ui.add_space(10.0);

            match self.current_tab {
                Tab::Steam => {
                    ui.label("2. Advanced (Optional):");
                    ui.horizontal(|ui| {
                        ui.label("Wait Process:");
                        ui.text_edit_singleline(&mut self.wait_process).on_hover_text("e.g. witcher3.exe");
                        if ui.button("🔍").on_hover_text("Pick from running processes").clicked() {
                             self.refresh_process_list();
                             self.show_process_picker = true;
                        }
                    });

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(15.0);

                    ui.label("3. Output:");
                    ui.add(egui::TextEdit::singleline(&mut self.generated_cmd).interactive(false));
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Generate Command").clicked() { self.generate_steam_command(); }
                        if ui.button("Copy").clicked() { self.copy_to_clipboard(); }
                    });
                },
                Tab::EpicWrapper => {
                    ui.label("2. Wrapper Installation:");
                    ui.label(egui::RichText::new("Automated setup for Epic/GOG/Standalone games.").size(11.0));
                    ui.add_space(5.0);

                    if ui.button("Browse Game Executable...").clicked() {
                        self.browse_game_executable();
                    }
                    
                    if let Some(path) = &self.selected_game_path {
                         ui.label(egui::RichText::new(format!("Selected: {}", path)).size(10.0));
                    } else {
                         ui.label(egui::RichText::new("No file selected").italics());
                    }
                    ui.add_space(5.0);

                    ui.label("Renamed Target Name:");
                    ui.text_edit_singleline(&mut self.target_exe).on_hover_text("The original game executable will be renamed to this.");
                    
                    ui.label("Wait Process (Optional):");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.wait_process).on_hover_text("If the game uses a launcher that closes, put the actual game exe name here.");
                        if ui.button("🔍").on_hover_text("Pick from running processes").clicked() {
                             self.refresh_process_list();
                             self.show_process_picker = true;
                        }
                    });

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(15.0);
                    
                    let is_installed = self.check_is_installed();

                    if is_installed {
                         ui.label(egui::RichText::new("Wrapper appears to be INSTALLED.").color(egui::Color32::GREEN));
                         ui.add_space(5.0);
                         if ui.button(egui::RichText::new("Uninstall Wrapper").color(egui::Color32::RED)).clicked() {
                              self.uninstall_wrapper();
                         }
                    } else {
                         ui.label(egui::RichText::new("Warning: This will rename the selected file and replace it with this launcher.").color(egui::Color32::YELLOW));
                         ui.add_space(10.0);
                         if ui.button("Install Wrapper").clicked() {
                             self.install_wrapper();
                         }
                    }
                }
            }

            ui.add_space(10.0);
            if !self.status_msg.is_empty() {
                ui.label(egui::RichText::new(&self.status_msg).color(egui::Color32::GREEN));
            }
        });
    }
}