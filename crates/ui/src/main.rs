use core::types::database::Database;
use core::types::schema::{DbSchema, DbColumn, DbColumnType, DbValue};
use core::types::table::{Table, Row};
use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;

#[derive(Default)]
struct DatabaseApp {
    database: Option<Database>,
    database_path: Option<PathBuf>,
    selected_table: Option<String>,
    new_table_name: String,
    show_schema_window: bool,
    new_schema: Vec<DbColumn>,
    temp_column_name: String,
    temp_column_type: DbColumnType,
    new_db_name: String,
    has_unsaved_changes: bool,
    show_close_confirmation: bool,
    show_intersection_window: bool,
    intersection_table: Option<String>,
    intersection_result: Option<Vec<Row>>,
}

impl DatabaseApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            new_db_name: String::new(),
            has_unsaved_changes: false,
            show_close_confirmation: false,
            show_intersection_window: false,
            intersection_table: None,
            intersection_result: None,
            ..Default::default()
        }
    }

    fn mark_as_modified(&mut self) {
        self.has_unsaved_changes = true;
    }

    fn save_database(&mut self) -> bool {
        if let (Some(db), Some(path)) = (&self.database, &self.database_path) {
            if let Ok(json) = serde_json::to_string_pretty(db) {
                if std::fs::write(path, json).is_ok() {
                    self.has_unsaved_changes = false;
                    return true;
                }
            }
        }
        false
    }

    fn handle_close_confirmation(&mut self, ctx: &egui::Context) {
        if !self.show_close_confirmation {
            return;
        }

        egui::Window::new("Confirm Close")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading("Unsaved Changes");
                ui.label("You have unsaved changes. Do you want to save before closing?");
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    let mut action = None;
                    if ui.button("Save and Close").clicked() {
                        action = Some(true);
                    }
                    if ui.button("Close without Saving").clicked() {
                        action = Some(false);
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_close_confirmation = false;
                    }
                    
                    if let Some(save) = action {
                        if !save || self.save_database() {
                            self.close_database();
                        }
                    }
                });
            });
    }

    fn try_close_database(&mut self) {
        if self.has_unsaved_changes {
            self.show_close_confirmation = true;
        } else {
            self.close_database();
        }
    }

    fn close_database(&mut self) {
        self.database = None;
        self.database_path = None;
        self.selected_table = None;
        self.has_unsaved_changes = false;
        self.show_close_confirmation = false;
    }

    fn show_database_selection(&mut self, ui: &mut egui::Ui) {
        ui.heading("Database Management");
        
        // Create new database section
        ui.group(|ui| {
            ui.heading("Create New Database");
            ui.horizontal(|ui| {
                ui.label("Database name:");
                ui.text_edit_singleline(&mut self.new_db_name);
                if ui.button("Create").clicked() && !self.new_db_name.is_empty() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Database", &["json"])
                        .set_file_name(&format!("{}.json", self.new_db_name))
                        .save_file()
                    {
                        self.database = Some(Database::new(&self.new_db_name));
                        self.database_path = Some(path.clone());
                        self.save_database();
                        self.new_db_name.clear();
                    }
                }
            });
        });

        ui.add_space(10.0);

        // Open existing database section
        ui.group(|ui| {
            ui.heading("Open Existing Database");
            if ui.button("Open Database File").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("Database", &["json"])
                    .pick_file()
                {
                    self.database_path = Some(path.clone());
                    if let Ok(file_content) = std::fs::read_to_string(&path) {
                        if let Ok(db) = serde_json::from_str(&file_content) {
                            self.database = Some(db);
                            self.has_unsaved_changes = false;
                        }
                    }
                }
            }
        });

        // Current database info
        if let Some(path) = &self.database_path {
            ui.add_space(10.0);
            
            // Prepare all the data we need before entering the UI closure
            let path_display = path.display().to_string();
            let has_changes = self.has_unsaved_changes;
            let db_info = self.database.as_ref().map(|db| (db.name.clone(), db.tables.len()));
            
            ui.group(|ui| {
                ui.heading("Current Database");
                ui.horizontal(|ui| {
                    if has_changes {
                        ui.label("⚠ Unsaved changes");
                        if ui.button("Save").clicked() {
                            self.save_database();
                        }
                    }
                    if ui.button("Close Database").clicked() {
                        self.try_close_database();
                    }
                });
                ui.label(format!("Path: {}", path_display));
                if let Some((name, table_count)) = db_info {
                    ui.label(format!("Name: {}", name));
                    ui.label(format!("Tables: {}", table_count));
                }
            });
        }
    }

    fn show_tables_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Tables");
            if ui.button("← Back to Database").clicked() {
                self.try_close_database();
            }
        });
        
        // New table creation
        ui.horizontal(|ui| {
            ui.label("New table name:");
            ui.text_edit_singleline(&mut self.new_table_name);
            if ui.button("Create Table").clicked() && !self.new_table_name.is_empty() {
                self.show_schema_window = true;
            }
        });

        // Tables list
        if let Some(db) = &self.database {
            let table_names: Vec<_> = db.tables.iter().map(|t| t.name().to_string()).collect();
            for table_name in table_names {
                let table_name_clone = table_name.clone();
                ui.horizontal(|ui| {
                    if ui.button(&table_name).clicked() {
                        self.selected_table = Some(table_name.clone());
                    }
                    if ui.button("🗑").clicked() {
                        if let Some(db) = &mut self.database {
                            db.delete_table(&table_name_clone);
                            if self.selected_table.as_deref() == Some(&table_name_clone) {
                                self.selected_table = None;
                            }
                            self.mark_as_modified();
                        }
                    }
                });
            }
        }
    }

    fn show_schema_window(&mut self, ctx: &egui::Context) {
        if !self.show_schema_window {
            return;
        }

        let mut close_window = false;

        egui::Window::new("Define Schema")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Define Table Schema");
                    if ui.button("✕").clicked() {
                        close_window = true;
                    }
                });

                // Option to copy schema from existing table
                if let Some(db) = &self.database {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Copy schema from existing table:").strong());
                        egui::ScrollArea::vertical()
                            .id_source("schema_copy_scroll")
                            .max_height(150.0)
                            .show(ui, |ui| {
                                for table in &db.tables {
                                    ui.horizontal(|ui| {
                                        if ui.button(table.name()).clicked() {
                                            self.new_schema = table.schema.columns.clone();
                                        }
                                        // Show schema preview
                                        ui.label("→");
                                        for col in &table.schema.columns {
                                            ui.label(format!("{} ({})", col.name, format!("{:?}", col.column_type)));
                                        }
                                    });
                                }
                            });
                    });
                    ui.separator();
                }

                // Add new column
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Add columns manually:").strong());
                    ui.horizontal(|ui| {
                        ui.label("Column name:");
                        let text_edit = ui.text_edit_singleline(&mut self.temp_column_name);
                        egui::ComboBox::from_label("Type")
                            .selected_text(format!("{:?}", self.temp_column_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.temp_column_type, DbColumnType::Integer, "Integer");
                                ui.selectable_value(&mut self.temp_column_type, DbColumnType::Real, "Real");
                                ui.selectable_value(&mut self.temp_column_type, DbColumnType::String, "String");
                                ui.selectable_value(&mut self.temp_column_type, DbColumnType::Char, "Char");
                                ui.selectable_value(&mut self.temp_column_type, DbColumnType::Money, "Money");
                                ui.selectable_value(&mut self.temp_column_type, DbColumnType::MoneyRange, "Money Range");
                            });
                        if (ui.button("Add Column").clicked() || text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) 
                            && !self.temp_column_name.is_empty() {
                            self.new_schema.push(DbColumn {
                                name: self.temp_column_name.clone(),
                                column_type: self.temp_column_type.clone(),
                            });
                            self.temp_column_name.clear();
                        }
                    });
                });

                // Show current schema
                if !self.new_schema.is_empty() {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Current Schema:").strong());
                        let mut to_remove = None;
                        for (i, col) in self.new_schema.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: {:?}", col.name, col.column_type));
                                if ui.button("Remove").clicked() {
                                    to_remove = Some(i);
                                }
                            });
                        }
                        if let Some(i) = to_remove {
                            self.new_schema.remove(i);
                        }
                    });
                }

                ui.separator();

                // Buttons at the bottom
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        close_window = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_create = !self.new_schema.is_empty() && !self.new_table_name.is_empty();
                        if ui.add_enabled(can_create, egui::Button::new("Create Table")).clicked() {
                            if let Some(db) = &mut self.database {
                                let schema = DbSchema {
                                    columns: self.new_schema.clone(),
                                };
                                let table = Table::new(self.new_table_name.clone(), schema);
                                db.add_table(table);
                                self.mark_as_modified();
                                close_window = true;
                            }
                        }
                        if !can_create {
                            ui.label(egui::RichText::new("Table name and at least one column are required").color(egui::Color32::RED));
                        }
                    });
                });
            });

        if close_window {
            self.show_schema_window = false;
            self.new_schema.clear();
            self.new_table_name.clear();
        }
    }

    fn show_intersection_window(&mut self, ctx: &egui::Context) {
        if !self.show_intersection_window {
            return;
        }

        let mut close_window = false;
        let mut new_intersection_table = None;
        let mut error_message = None;

        egui::Window::new("Table Intersection")
            .collapsible(false)
            .resizable(true)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Select Table for Intersection");
                    if ui.button("✕").clicked() {
                        close_window = true;
                    }
                });

                if let Some(db) = &self.database {
                    if let Some(current_table) = &self.selected_table {
                        let table_names: Vec<_> = db.tables.iter()
                            .map(|t| t.name().to_string())
                            .filter(|name| name != current_table)
                            .collect();

                        // Show current table schema
                        if let Some(current) = db.get_table(current_table) {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Current Table Schema:").strong());
                                ui.label(format!("Table: {}", current_table));
                                for col in &current.schema.columns {
                                    ui.label(format!("  {} ({})", col.name, format!("{:?}", col.column_type)));
                                }
                            });
                        }

                        ui.separator();
                        ui.label(egui::RichText::new("Select a table with matching schema:").strong());
                        
                        // Show available tables with schema preview
                        egui::ScrollArea::vertical()
                            .id_source("intersection_tables_scroll")
                            .show(ui, |ui| {
                                for table_name in &table_names {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            if let Some(table) = db.get_table(table_name) {
                                                if ui.button(table_name).clicked() {
                                                    if let Some(table1) = db.get_table(current_table) {
                                                        // Compare only column types, ignoring schema names
                                                        if table1.schema.columns.len() != table.schema.columns.len() {
                                                            error_message = Some("Tables have different number of columns".to_string());
                                                        } else {
                                                            let schemas_match = table1.schema.columns.iter().zip(&table.schema.columns)
                                                                .all(|(c1, c2)| c1.column_type == c2.column_type);
                                                            
                                                            if !schemas_match {
                                                                error_message = Some("Column types do not match".to_string());
                                                            } else {
                                                                match table1.intersection(table) {
                                                                    Ok(result) => {
                                                                        println!("Found intersection with {} rows", result.len());
                                                                        self.intersection_result = Some(result);
                                                                        self.intersection_table = Some(table_name.clone());
                                                                        error_message = None;
                                                                    }
                                                                    Err(e) => {
                                                                        error_message = Some(format!("Error: {}", e));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                ui.vertical(|ui| {
                                                    for col in &table.schema.columns {
                                                        ui.label(format!("{} ({})", col.name, format!("{:?}", col.column_type)));
                                                    }
                                                });
                                            }
                                        });
                                    });
                                }
                            });

                        // Show error message if any
                        if let Some(error) = &error_message {
                            ui.separator();
                            ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                        }

                        ui.separator();

                        if let (Some(result), Some(other_table)) = (&self.intersection_result, &self.intersection_table) {
                            ui.heading(format!("Intersection with {}", other_table));
                            
                            if let Some(table) = db.get_table(current_table) {
                                // Create a scrollable area for the results
                                egui::ScrollArea::both()
                                    .id_source("intersection_results_scroll")
                                    .show(ui, |ui| {
                                        // Show header
                                        ui.horizontal(|ui| {
                                            for col in &table.schema.columns {
                                                ui.label(egui::RichText::new(&col.name).strong());
                                            }
                                        });

                                        // Show intersection rows
                                        for row in result {
                                            ui.horizontal(|ui| {
                                                for value in &row.values {
                                                    let text = match value {
                                                        DbValue::Integer(n) => n.to_string(),
                                                        DbValue::Real(n) => format!("{:.2}", n),
                                                        DbValue::String(s) => s.clone(),
                                                        DbValue::Char(c) => c.to_string(),
                                                        DbValue::Money(m) => format!("${:.2}", m),
                                                        DbValue::MoneyRange(start, end) => format!("${:.2}-${:.2}", start, end),
                                                    };
                                                    ui.label(text);
                                                }
                                            });
                                        }

                                        if result.is_empty() {
                                            ui.label(egui::RichText::new("No matching rows found").color(egui::Color32::RED));
                                        } else {
                                            ui.label(format!("Found {} matching rows", result.len()));
                                        }
                                    });

                                // Add a button to save intersection as a new table
                                if !result.is_empty() {
                                    ui.separator();
                                    if ui.button("Save as New Table").clicked() {
                                        let new_table_name = format!("{}_{}_intersection", current_table, other_table);
                                        let schema = DbSchema {
                                            columns: table.schema.columns.clone(),
                                        };
                                        let mut new_table = Table::new(new_table_name.clone(), schema);
                                        for row in result {
                                            let _ = new_table.insert(row.values.clone());
                                        }
                                        new_intersection_table = Some(new_table);
                                        close_window = true;
                                    }
                                }
                            }
                        }
                    }
                }
            });

        // Handle the new table creation outside the UI closure
        if let Some(new_table) = new_intersection_table {
            if let Some(db) = &mut self.database {
                db.add_table(new_table);
                self.mark_as_modified();
            }
        }

        if close_window {
            self.show_intersection_window = false;
            self.intersection_result = None;
            self.intersection_table = None;
        }
    }

    fn show_table_view(&mut self, ui: &mut egui::Ui) {
        if let Some(table_name) = &self.selected_table.clone() {
            if let Some(db) = &mut self.database {
                if let Some(table) = db.get_table_mut(table_name) {
                    let mut go_back = false;
                    let mut add_row = false;
                    let schema = table.schema.clone();
                    
                    ui.horizontal(|ui| {
                        if ui.button("← Back to Tables").clicked() {
                            go_back = true;
                        }
                        ui.heading(table_name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Find Intersection").clicked() {
                                self.show_intersection_window = true;
                                self.intersection_result = None;
                                self.intersection_table = None;
                            }
                            ui.add_space(8.0);
                            if ui.button("Add Row").clicked() {
                                add_row = true;
                            }
                        });
                    });

                    if go_back {
                        self.selected_table = None;
                        return;
                    }

                    // Table header
                    ui.horizontal(|ui| {
                        for col in &schema.columns {
                            ui.label(&col.name);
                        }
                        ui.label("Actions");
                    });

                    // Table rows
                    let mut rows: Vec<_> = table.rows.iter().collect();
                    rows.sort_by_key(|(id, _)| **id);
                    
                    let mut to_delete = None;
                    let mut updates = Vec::new();

                    for (&id, row) in &rows {
                        let mut new_values = row.values.clone();
                        let mut changed = false;

                        ui.horizontal(|ui| {
                            for (value, _col) in new_values.iter_mut().zip(&schema.columns) {
                                match value {
                                    DbValue::Integer(n) => {
                                        let mut text = n.to_string();
                                        if ui.text_edit_singleline(&mut text).changed() {
                                            if let Ok(new_val) = text.parse() {
                                                *n = new_val;
                                                changed = true;
                                            }
                                        }
                                    }
                                    DbValue::Real(n) => {
                                        let mut text = n.to_string();
                                        if ui.text_edit_singleline(&mut text).changed() {
                                            if let Ok(new_val) = text.parse() {
                                                *n = new_val;
                                                changed = true;
                                            }
                                        }
                                    }
                                    DbValue::String(s) => {
                                        let mut text = s.clone();
                                        if ui.text_edit_singleline(&mut text).changed() {
                                            *s = text;
                                            changed = true;
                                        }
                                    }
                                    DbValue::Char(c) => {
                                        let mut text = c.to_string();
                                        if ui.text_edit_singleline(&mut text).changed() && !text.is_empty() {
                                            if let Some(new_char) = text.chars().next() {
                                                *c = new_char;
                                                changed = true;
                                            }
                                        }
                                    }
                                    DbValue::Money(m) => {
                                        let mut text = m.to_string();
                                        if ui.text_edit_singleline(&mut text).changed() {
                                            if let Ok(new_val) = text.parse() {
                                                *m = new_val;
                                                changed = true;
                                            }
                                        }
                                    }
                                    DbValue::MoneyRange(start, end) => {
                                        ui.horizontal(|ui| {
                                            let mut start_text = start.to_string();
                                            let mut end_text = end.to_string();
                                            if ui.text_edit_singleline(&mut start_text).changed() {
                                                if let Ok(new_val) = start_text.parse() {
                                                    *start = new_val;
                                                    changed = true;
                                                }
                                            }
                                            ui.label("-");
                                            if ui.text_edit_singleline(&mut end_text).changed() {
                                                if let Ok(new_val) = end_text.parse() {
                                                    *end = new_val;
                                                    changed = true;
                                                }
                                            }
                                        });
                                    }
                                }
                            }

                            if ui.button("🗑").clicked() {
                                to_delete = Some(id);
                            }
                        });

                        if changed {
                            updates.push((id, new_values));
                        }
                    }

                    // Apply updates
                    let mut modified = false;
                    if let Some(id) = to_delete {
                        if table.delete(id).is_ok() {
                            modified = true;
                        }
                    }

                    for (id, values) in updates {
                        if table.update(id, values).is_ok() {
                            modified = true;
                        }
                    }

                    if add_row {
                        let new_row = schema.columns.iter().map(|col| {
                            match col.column_type {
                                DbColumnType::Integer => DbValue::Integer(0),
                                DbColumnType::Real => DbValue::Real(0.0),
                                DbColumnType::String => DbValue::String(String::new()),
                                DbColumnType::Char => DbValue::Char(' '),
                                DbColumnType::Money => DbValue::Money(0.0),
                                DbColumnType::MoneyRange => DbValue::MoneyRange(0.0, 0.0),
                            }
                        }).collect();
                        if table.insert(new_row).is_ok() {
                            modified = true;
                        }
                    }

                    if modified {
                        self.mark_as_modified();
                    }
                }
            }
        }
    }
}

impl eframe::App for DatabaseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.show_intersection_window {
                self.show_intersection_window = false;
                self.intersection_result = None;
                self.intersection_table = None;
            } else if self.show_schema_window {
                self.show_schema_window = false;
                self.new_schema.clear();
                self.new_table_name.clear();
            } else if self.selected_table.is_some() {
                self.selected_table = None;
            } else if self.database.is_some() {
                self.try_close_database();
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let has_changes = self.has_unsaved_changes;
                if self.database.is_some() {
                    if has_changes {
                        ui.label("⚠ Unsaved changes");
                        if ui.button("Save").clicked() {
                            self.save_database();
                        }
                    }
                    if ui.button("Close Database").clicked() {
                        self.try_close_database();
                    }
                }
            });
        });

        self.handle_close_confirmation(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.database.is_none() {
                self.show_database_selection(ui);
            } else {
                egui::SidePanel::left("tables_list").show(ctx, |ui| {
                    self.show_tables_list(ui);
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    self.show_table_view(ui);
                });
            }
        });

        if self.show_schema_window {
            self.show_schema_window(ctx);
        }

        if self.show_intersection_window {
            self.show_intersection_window(ctx);
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Database Manager",
        options,
        Box::new(|cc| Box::new(DatabaseApp::new(cc)))
    )
}
