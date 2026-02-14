use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Entry, Orientation, Label, ScrolledWindow, CheckButton, HeaderBar, Separator};
use gtk4::glib; // Correct glib import
use crate::db;
use crate::sync::Task;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::config::Config;
use crate::sync_manager::SyncCommand;

pub fn build_ui(app: &Application, cmd_tx: Sender<SyncCommand>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Go2Do")
        .default_width(370) // Increased to show header title
        .default_height(250)
        .build();

    // Standard HeaderBar
    let header = HeaderBar::new();
    header.set_show_title_buttons(true); 
    let header = HeaderBar::new();
    header.set_show_title_buttons(true); 
    
    // Sync Indicator
    // Use a Label with a bullet point or space to ensure it renders context
    let sync_dot = Label::new(Some("●")); 
    sync_dot.add_css_class("sync-status-dot");
    sync_dot.set_widget_name("sync_indicator"); 
    sync_dot.set_tooltip_text(Some("Sync Status: Idle"));
    header.pack_start(&sync_dot); // Moved to start (Left)

    window.set_titlebar(Some(&header));


    let main_box = Box::new(Orientation::Vertical, 0); // No spacing here, manage manually
    main_box.add_css_class("main-content");

    // Visual Barrier (Horizontal Line)
    let separator = Separator::new(Orientation::Horizontal);
    separator.add_css_class("header-separator");
    main_box.append(&separator);
    
    // Input Area
    let entry = Entry::builder()
        .placeholder_text("Add a new task...")
        .build();
    
    main_box.append(&entry);

    // Task List Container
    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_height(250)
        .vexpand(true)
        .build();

    let task_list_box = Box::new(Orientation::Vertical, 5);
    task_list_box.add_css_class("task-list-container");
    scrolled_window.set_child(Some(&task_list_box));
    main_box.append(&scrolled_window);

    // Toggle Show Completed Button
    let show_completed_btn = gtk4::ToggleButton::builder()
        .label("Show Done")
        .has_frame(false) // Flat style
        .active(false)
        .build();
    
    // Pack end order: First added is overflow/rightmost? 
    // Usually pack_end adds from right to left.
    // So if we want [Dot] [Button], and we add Button first, it is rightmost.
    // Then we add Dot, it is to the left of Button.
    // Pack start: [Sync Dot] [Show Done] ... [Title] ...
    header.pack_start(&show_completed_btn);
    // header.pack_end(&sync_dot); // Moved to start

    window.set_child(Some(&main_box));

    // Refresh Logic
    let task_list_box_rc = Rc::new(task_list_box);
    let show_completed_btn_rc = Rc::new(show_completed_btn);
    
    let task_list_box_for_refresh = task_list_box_rc.clone();
    let show_completed_for_refresh = show_completed_btn_rc.clone();

    let refresh_ui = Rc::new(move || {
        let task_list_box = &task_list_box_for_refresh;
        let show_completed = show_completed_for_refresh.is_active();
        
        // clear existing
        while let Some(child) = task_list_box.first_child() {
            task_list_box.remove(&child);
        }

        // fetch from db
        println!("Storage: Reading tasks from LOCAL database.");
        let conn = db::init().expect("failed to init db");
        let mut tasks = db::get_tasks(&conn).unwrap_or_default();

        // Sort: Uncompleted first, then by created_at (newest first).
        tasks.sort_by(|a, b| {
            match (a.completed, b.completed) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => b.created_at.cmp(&a.created_at), // Newest first
            }
        });
        
        // Split into active and completed
        let (completed, active): (Vec<Task>, Vec<Task>) = tasks.into_iter().partition(|t| t.completed);

        // Render Active
        for task in active {
            let row = create_task_row(&task);
            task_list_box.append(&row);
        }

        // Render Completed if toggled
        if show_completed && !completed.is_empty() {
             // Separator
             let sep_box = Box::new(Orientation::Horizontal, 0);
             sep_box.add_css_class("completed-section-header");
             let sep_label = Label::new(Some("Completed"));
             sep_label.add_css_class("completed-section-label");
             sep_box.append(&sep_label);
             task_list_box.append(&sep_box);

             for task in completed {
                 let row = create_task_row(&task);
                 task_list_box.append(&row);
             }
        }
    });

    // Initial Load
    refresh_ui();

    // Wire up Toggle Button
    let refresh_for_toggle = refresh_ui.clone();
    show_completed_btn_rc.connect_toggled(move |_| {
        refresh_for_toggle();
    });

    // Setup Periodic Cleanup Timer (Every 60 seconds)
    // Note: This cleanup removes old completed tasks. 
    // If user wants to see them, maybe we should relax this? 
    // For now, keeping as is (1 min retention for completed).
    let refresh_for_timer = refresh_ui.clone();
    glib::timeout_add_seconds_local(60, move || {
        // Run cleanup
        if let Ok(conn) = db::init() {
           if let Ok(count) = db::soft_delete_old_completed_tasks(&conn) {
               if count > 0 {
                   println!("Cleanup: Removed {} old completed tasks.", count);
                   refresh_for_timer();
               }
           }
        }
        glib::ControlFlow::Continue
    });

    // Handle Entry Enter
    let vadjustment = scrolled_window.vadjustment(); // Get adjustment for scrolling
    let refresh_for_add = refresh_ui.clone(); // Full refresh on add to insure correct order
    
    entry.connect_activate(move |e| {
        let text = e.text();
        if text.is_empty() { return; }

        let conn = db::init().unwrap();
        let new_task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "dev-user".to_string(), // placeholder
            created_at: chrono::Utc::now().timestamp_millis(),
            updated_at: chrono::Utc::now().timestamp_millis(),
            deleted_at: None,
            content: text.to_string(),
            completed: false,
            pinned: false,
            version: 1,
            device_id: "desktop-v1".to_string(),
        };

        if let Ok(_) = db::create_task(&conn, &new_task) {
             e.set_text("");
             refresh_for_add(); // Refresh to show new task
             
             // Scroll to top
             vadjustment.set_value(0.0);
        }
    });
    
    // Quick Input: Focus Entry on Show
    let entry_clone = entry.clone();
    window.connect_show(move |_| {
        entry_clone.grab_focus();
    });

    // Close on Escape
    let key_controller = gtk4::EventControllerKey::new();
    let window_close = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            window_close.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(key_controller);

    window.present();
}

pub fn show_login_dialog(window: &ApplicationWindow, cmd_tx: Sender<SyncCommand>) {
    let dialog = gtk4::Window::builder()
        .transient_for(window)
        .modal(true)
        .title("Login to Sync")
        .default_width(300)
        .default_height(200)
        .build();

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);

    let email_entry = gtk4::Entry::builder()
        .placeholder_text("Email")
        .build();
    
    let password_entry = gtk4::Entry::builder()
        .placeholder_text("Password")
        .visibility(false)
        .build();
    
    let login_btn = gtk4::Button::builder()
        .label("Login / Connect")
        .css_classes(vec!["suggested-action"])
        .build();
    
    let status_label = gtk4::Label::new(Some("Enter your credentials"));
    status_label.set_css_classes(&["dim-label"]);

    vbox.append(&email_entry);
    vbox.append(&password_entry);
    vbox.append(&login_btn);
    vbox.append(&status_label);

    dialog.set_child(Some(&vbox));

    let email_buffer = email_entry.buffer();
    let pass_buffer = password_entry.buffer();
    let dialog_weak = dialog.downgrade();

    login_btn.connect_clicked(move |_| {
        let email = email_buffer.text();
        let password = pass_buffer.text();
        
        if email.is_empty() || password.is_empty() {
            return;
        }

        // Send Login Command
        let _ = cmd_tx.send(SyncCommand::Login(email.to_string(), password.to_string()));
        
        // Close dialog
        if let Some(dialog) = dialog_weak.upgrade() {
            dialog.close();
        }
    });

    dialog.present();
}

pub fn show_settings_dialog(window: &ApplicationWindow, current_url: String, cmd_tx: Sender<SyncCommand>) {
    let dialog = gtk4::Window::builder()
        .title("Server Settings")
        .transient_for(window)
        .modal(true)
        .default_width(400)
        .resizable(false)
        .build();

    let vbox = Box::new(Orientation::Vertical, 12);
    vbox.set_margin_top(18);
    vbox.set_margin_bottom(18);
    vbox.set_margin_start(18);
    vbox.set_margin_end(18);

    let label = gtk4::Label::new(Some("Server API URL"));
    label.set_halign(gtk4::Align::Start);
    vbox.append(&label);

    let entry = gtk4::Entry::new();
    entry.set_text(&current_url);
    vbox.append(&entry);

    let help_label = gtk4::Label::new(Some("e.g. https://go2do-server.<your-subdomain>.workers.dev"));
    help_label.add_css_class("caption");
    help_label.set_halign(gtk4::Align::Start);
    vbox.append(&help_label);

    let save_btn = gtk4::Button::with_label("Save & Apply");
    save_btn.add_css_class("suggested-action");
    vbox.append(&save_btn);

    dialog.set_child(Some(&vbox));

    let dialog_weak = dialog.downgrade();
    save_btn.connect_clicked(move |_| {
        let new_url = entry.text().to_string();
        if new_url.is_empty() { return; }

        // Save to config file
        let mut cfg = Config::load();
        cfg.api_url = new_url.clone();
        if let Err(e) = cfg.save() {
            println!("UI: Failed to save config: {}", e);
        }

        // Send Update Command to SyncManager
        let _ = cmd_tx.send(SyncCommand::UpdateUrl(new_url));
        
        // Close dialog
        if let Some(d) = dialog_weak.upgrade() {
            d.close();
        }
    });

    dialog.present();
}

pub fn show_error_dialog(window: &ApplicationWindow, message: &str) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .message_type(gtk4::MessageType::Error)
        .buttons(gtk4::ButtonsType::Ok)
        .text("Login Failed")
        .secondary_text(message)
        .build();
    
    dialog.connect_response(|d, _| {
        d.close();
    });
    
    dialog.present();
}

pub fn show_help_dialog(window: &ApplicationWindow) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .message_type(gtk4::MessageType::Info)
        .buttons(gtk4::ButtonsType::Ok)
        .text("Go2Do Shortcuts")
        .secondary_text("
Ctrl + N: Add New Task
Esc: Hide Window
Ctrl + R: Force Sync
Ctrl + Q: Quit app
        ")
        .build();
    
    dialog.connect_response(|d, _| {
        d.close();
    });
    
    dialog.present();
}

// Helper to create a consistent task row
fn create_task_row(task: &Task) -> Box {
    let row = Box::new(Orientation::Horizontal, 4); // Reduced spacing
    row.add_css_class("task-row");
    
    let check = CheckButton::builder()
        .active(task.completed)
        .build();
    let label = Label::new(Some(&task.content));
    
    // Add sorting/styling class
    if task.completed {
        label.add_css_class("completed-label");
        check.add_css_class("completed-check");
    }
    
    row.append(&check);
    row.append(&label);
    
    let task_id = task.id.clone();
    
    // Toggle Logic
    // We cannot easily refresh the WHOLE UI from here without passing the refresh closure down.
    // Instead, we just update DB and Visuals locally.
    // The next refresh (toggle, add, timer) will fix order.
    check.connect_toggled(move |c| {
        let conn = db::init().unwrap();
        let _ = db::update_task_completion(&conn, &task_id, c.is_active());
        
        // Visual update
        if c.is_active() {
            label.add_css_class("completed-label");
        } else {
            label.remove_css_class("completed-label");
        }
    });

    // Row Click Gesture (Make label clickable)
    let gesture = gtk4::GestureClick::new();
    let check_clone_2 = check.clone();
    gesture.connect_pressed(move |_, _, _, _| {
        // Toggle the checkbutton, which triggers connect_toggled
        check_clone_2.set_active(!check_clone_2.is_active());
    });
    row.add_controller(gesture);

    // Set pointer cursor to indicate clickability
    let cursor = gtk4::gdk::Cursor::from_name("pointer", None);
    row.set_cursor(cursor.as_ref());
    check.set_cursor(cursor.as_ref());

    row
}

