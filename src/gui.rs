use eframe::egui;
use std::env;
use arboard::Clipboard;
use crate::backends::get_backend;

pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 450.0]) // Slightly taller for the new field
            .with_resizable(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "Steam HDR Launcher",
        options,
        // Wrap the closure in Ok() as required by newer eframe versions
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    monitors: Vec<String>,
    selected_monitor: Option<String>,
    wait_process: String, // New field for the --wait flag
    generated_cmd: String,
    status_msg: String,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 1. Scan monitors on startup
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
            generated_cmd: String::new(),
            status_msg: String::new(),
        }
    }

    fn generate_command(&mut self) {
        if let Some(monitor) = &self.selected_monitor {
            if let Ok(exe_path) = env::current_exe() {
                let exe_str = exe_path.to_string_lossy();
                
                // Logic to add --wait if the user typed something
                let wait_arg = if !self.wait_process.trim().is_empty() {
                    format!("--wait \"{}\" ", self.wait_process.trim())
                } else {
                    String::new()
                };

                // Format: "Path\To\Exe" "Monitor Name" --wait "witcher3.exe" -- %command%
                self.generated_cmd = format!("\"{}\" \"{}\" {}-- %command%", exe_str, monitor, wait_arg);
                self.status_msg = "Command generated!".to_string();
            }
        }
    }

    fn copy_to_clipboard(&mut self) {
        if self.generated_cmd.is_empty() {
            return;
        }
        
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
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Steam HDR Launcher Generator");
            ui.add_space(10.0);

            // --- SECTION 1: MONITOR ---
            ui.label("1. Select your HDR Monitor:");
            
            let selected_text = self.selected_monitor.as_deref().unwrap_or("No monitors found");
            
            egui::ComboBox::from_id_salt("monitor_combo")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for monitor in &self.monitors {
                        ui.selectable_value(&mut self.selected_monitor, Some(monitor.clone()), monitor);
                    }
                });

            if ui.button("Refresh Monitors").clicked() {
                let backend = get_backend();
                if let Ok(new_list) = backend.get_monitors() {
                    self.monitors = new_list;
                    if !self.monitors.is_empty() {
                        self.selected_monitor = Some(self.monitors[0].clone());
                    }
                }
            }

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // --- SECTION 2: SPECIAL HANDLING (Witcher 3, etc) ---
            ui.label("2. Special Handling (Optional):");
            ui.label(egui::RichText::new("Use this if the game has a launcher that closes immediately (e.g., Witcher 3, Cyberpunk).").size(10.0).color(egui::Color32::GRAY));
            
            ui.horizontal(|ui| {
                ui.label("Wait for Process:");
                ui.add(egui::TextEdit::singleline(&mut self.wait_process).hint_text("e.g. witcher3.exe"));
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // --- SECTION 3: OUTPUT ---
            ui.label("3. Steam Launch Option:");
            
            ui.add(
                egui::TextEdit::singleline(&mut self.generated_cmd)
                    .hint_text("Click Generate to create command...")
                    .interactive(false) // Read only
            );

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Generate Command").clicked() {
                    self.generate_command();
                }
                
                if ui.button("Copy to Clipboard").clicked() {
                    self.copy_to_clipboard();
                }
            });

            ui.add_space(10.0);
            
            // Status Message
            if !self.status_msg.is_empty() {
                ui.label(egui::RichText::new(&self.status_msg).color(egui::Color32::GREEN));
            }
        });
    }
}