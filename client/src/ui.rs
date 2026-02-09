use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Entry, Orientation, Label, ScrolledWindow, CheckButton, HeaderBar, Separator};
use gtk4::glib; // Correct glib import
use crate::db;
use crate::sync::Task;
use std::rc::Rc;

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Go2Do")
        .default_width(325)
        .default_height(250)
        .build();

    // Standard HeaderBar
    let header = HeaderBar::new();
    header.set_show_title_buttons(true); 
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

    window.set_child(Some(&main_box));

    // Refresh Logic
    let task_list_box_rc = Rc::new(task_list_box);
    
    // Create a shared closure for refreshing
    // We need to move this into a Rc to be shared between the timer and the initial load
    // But closures with moves are hard to share.
    // Let's refactor: "load_tasks" function that takes the list_box.
    
    // Actually, simply cloning the logic is fine for now or using a helper function.
    // For the timer, we need a way to invoke refresh.
    
    // Let's define a separate function `reload_tasks` outside or make `refresh_ui` reusable?
    // Since `refresh_ui` captures `task_list_box_rc`, we can clone that.
    
    let task_list_box_for_refresh = task_list_box_rc.clone();
    let refresh_ui = Rc::new(move || {
        let task_list_box = &task_list_box_for_refresh;
        
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

        for task in tasks {
            let row = create_task_row(&task);
            task_list_box.append(&row);
        }
    });

    // Initial Load
    refresh_ui();

    // Setup Periodic Cleanup Timer (Every 60 seconds)
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
             
             // Use the helper to create the row, ensuring consistent behavior
             let row = create_task_row(&new_task);
             task_list_box_rc.prepend(&row); // add to top
             
             // Scroll to top
             vadjustment.set_value(0.0);
        }
    });

    window.present();
}

// Helper to create a consistent task row
fn create_task_row(task: &Task) -> Box {
    let row = Box::new(Orientation::Horizontal, 10);
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
    check.connect_toggled(move |c| {
        let conn = db::init().unwrap();
        let _ = db::update_task_completion(&conn, &task_id, c.is_active());
        // Visual update only for now
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

