mod db;
mod sync; // Keep for legacy/types if needed, or remove later
mod auth;
mod sync_manager;
mod ui;
mod config;

use gtk4::prelude::*;
use gtk4::Application;
use gtk4::glib; // Correct import via gtk4
use config::Config;


use crate::sync_manager::{SyncCommand, SyncMsg};

enum TrayMsg {
    Show,
    TriggerSync,
    ShowLogin,
    ShowSettings,
    ShowHelp, // New
    Quit,
}

// Removed local SyncCommand/SyncMsg definitions

struct TrayHandler {
    sender: glib::Sender<TrayMsg>,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for TrayHandler {
    fn icon_name(&self) -> String {
        "accessories-text-editor".into()
    }
    
    fn id(&self) -> String {
        "go2do".into()
    }

    // Left-click action
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(TrayMsg::Show);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Add Task".into(),
                activate: Box::new(|sender: &mut TrayHandler| {
                    let _ = sender.sender.send(TrayMsg::Show);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Login / Connect".into(),
                activate: Box::new(|sender: &mut TrayHandler| {
                    let _ = sender.sender.send(TrayMsg::ShowLogin);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Server Settings".into(),
                activate: Box::new(|sender: &mut TrayHandler| {
                    let _ = sender.sender.send(TrayMsg::ShowSettings);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Help & Shortcuts".into(),
                activate: Box::new(|sender: &mut TrayHandler| {
                    let _ = sender.sender.send(TrayMsg::ShowHelp);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Sync Now".into(),
                activate: Box::new(|sender: &mut TrayHandler| {
                    let _ = sender.sender.send(TrayMsg::TriggerSync);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_sender: &mut TrayHandler| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn main() {
    let app = Application::builder()
        .application_id("com.ideneyesa.go2do")
        .build();

    // Create a channel to communicate between Tray (thread) and GTK (main loop)
    let (tx, rx) = glib::MainContext::channel(glib::Priority::DEFAULT);
    
    // Sync Channel (Status updates FROM sync thread)
    let (sync_tx, sync_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

    // Sync Command Channel (Commands TO sync thread)
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<SyncCommand>();
    // let cmd_tx_tray = cmd_tx.clone(); // Removed unused clone
    
    // Start Tray Service (Linux Only)
    #[cfg(target_os = "linux")]
    {
        let service = ksni::TrayService::new(TrayHandler { sender: tx.clone() });
        let _tray_handle = service.handle();
        service.spawn();
    }



    app.connect_startup(|_| {
        let provider = gtk4::CssProvider::new();
        
        // Try to load from user-local share dir first (install location)
        // Try to load from user-local share dir first (install location)
        let home = std::env::var("HOME").unwrap_or_default();
        let paths = vec![
            format!("{}/.local/share/go2do/style.css", home),
            "/usr/share/go2do/style.css".to_string(),
            "/usr/local/share/go2do/style.css".to_string(),
            "style.css".to_string(), // Dev fallback
        ];

        let mut loaded = false;
        for path_str in paths {
            let path = std::path::Path::new(&path_str);
            if path.exists() {
                 provider.load_from_path(path.to_str().unwrap());
                 println!("Loaded CSS from: {:?}", path);
                 loaded = true;
                 break;
            }
        }
        
        if !loaded {
             println!("Warning: No style.css found. UI may look unstyled.");
        }
        
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    // Start Sync Thread
    println!("Background: Initializing sync manager...");
    let config = Config::load();
    let sync_base_url = config.api_url.clone();
    
    std::thread::spawn(move || {
        let mut manager = sync_manager::SyncManager::new(sync_base_url);
        
        loop {
            // Notify: Syncing
            let _ = sync_tx.send(SyncMsg::Syncing);
            
            match manager.run_sync_cycle() {
                Ok(_) => {
                    let _ = sync_tx.send(SyncMsg::Synced);
                },
                Err(e) => {
                    let _ = sync_tx.send(SyncMsg::Error(e.to_string()));
                }
            }
            
            // Wait for 60s OR a force command
            // We use recv_timeout to implement the sleep + interrupt capability
            match cmd_rx.recv_timeout(std::time::Duration::from_secs(60)) {
                Ok(SyncCommand::ForceSync) => {
                    println!("Sync: Force sync triggered.");
                }
                Ok(SyncCommand::Login(email, password)) => {
                    println!("Sync: Attempting login for {}...", email);
                    match manager.login(&email, &password) {
                        Ok(_) => {
                            println!("Sync: Login successful!");
                            let _ = sync_tx.send(SyncMsg::LoginSuccess);
                            // Trigger sync immediately
                            let _ = manager.run_sync_cycle();
                        },
                        Err(e) => {
                            println!("Sync: Login failed: {}", e);
                            let _ = sync_tx.send(SyncMsg::LoginFailed(e.to_string()));
                        }
                    }
                }
                Ok(SyncCommand::UpdateUrl(url)) => {
                    println!("Sync: Updating API URL to {}...", url);
                    manager.set_base_url(url);
                }
                Err(_) => {
                    // Timeout (normal interval), loop continues
                }
            }
        }
    });

    let cmd_tx_clone = cmd_tx.clone();
    app.connect_activate(move |app| ui::build_ui(app, cmd_tx_clone.clone()));
    
    // Handle Tray Messages
    let app_weak_tray = app.downgrade();
    rx.attach(None, move |msg| {
        match msg {
            TrayMsg::Show => {
                if let Some(app) = app_weak_tray.upgrade() {
                    if let Some(window) = app.active_window() {
                        window.present();
                    } else {
                        // Re-create if closed? 
                        // GTK4 applications usually keep running if hold is true, 
                        // but here we just rely on active window. 
                        // Note: ui::build_ui might need calling if no window exists.
                        ui::build_ui(&app, cmd_tx.clone());
                    }
                }
            },
            TrayMsg::ShowLogin => {
                if let Some(app) = app_weak_tray.upgrade() {
                    if let Some(window) = app.active_window() {
                        let win = window.downcast::<gtk4::ApplicationWindow>().unwrap();
                        ui::show_login_dialog(&win, cmd_tx.clone());
                    }
                }
            },
            TrayMsg::ShowSettings => {
                if let Some(app) = app_weak_tray.upgrade() {
                    if let Some(window) = app.active_window() {
                        let win = window.downcast::<gtk4::ApplicationWindow>().unwrap();
                        let config = Config::load();
                        ui::show_settings_dialog(&win, config.api_url, cmd_tx.clone());
                    }
                }
            },
            TrayMsg::ShowHelp => {
                if let Some(app) = app_weak_tray.upgrade() {
                    if let Some(window) = app.active_window() {
                        let win = window.downcast::<gtk4::ApplicationWindow>().unwrap();
                        ui::show_help_dialog(&win);
                    }
                }
            },
            TrayMsg::TriggerSync => {
                println!("Tray: Requesting Manual Sync...");
                let _ = cmd_tx.send(SyncCommand::ForceSync);
            },
            TrayMsg::Quit => {
                std::process::exit(0);
            }
        }
        glib::ControlFlow::Continue
    });
    
    // Handle Sync Messages
    let app_weak = app.downgrade();
    sync_rx.attach(None, move |msg| {
        if let Some(app) = app_weak.upgrade() {
            if let Some(window) = app.active_window() {
                // Find grid/header? 
                // We need to find the widget named "sync_indicator"
                // GTK4 Traversal is a bit manual.
                
                // Helper to find child recursively (BFS)
                fn find_child_by_name(widget: &gtk4::Widget, name: &str) -> Option<gtk4::Widget> {
                     let mut sibling = widget.first_child();
                     while let Some(child) = sibling {
                         // println!("Searching: {:?} (name: {:?})", child, child.widget_name());
                         if child.widget_name() == name {
                             return Some(child);
                         }
                         if let Some(found) = find_child_by_name(&child, name) {
                             return Some(found);
                         }
                         sibling = child.next_sibling();
                     }
                     None
                }
                
                if let Some(titlebar) = window.titlebar() {
                     if let Some(indicator) = find_child_by_name(&titlebar, "sync_indicator") {
                          // Update Class
                          indicator.remove_css_class("syncing");
                          indicator.remove_css_class("synced");
                          indicator.remove_css_class("error");
                          
                          match msg {
                              SyncMsg::Syncing => {
                                  indicator.add_css_class("syncing");
                                  indicator.set_tooltip_text(Some("Syncing..."));
                              },
                              SyncMsg::Synced => {
                                  indicator.add_css_class("synced");
                                  indicator.set_tooltip_text(Some("Synced"));
                              },
                              SyncMsg::Error(e) => {
                                  indicator.add_css_class("error");
                                  indicator.set_tooltip_text(Some(&format!("Sync Error: {}", e)));
                              },
                              SyncMsg::LoginSuccess => {
                                  println!("UI: Login Successful");
                                  indicator.set_tooltip_text(Some("Logged In"));
                              },
                              SyncMsg::LoginFailed(e) => {
                                  println!("UI: Login Failed: {}", e);
                                  indicator.add_css_class("error");
                                  indicator.set_tooltip_text(Some(&format!("Login Failed: {}", e)));
                                  let win = window.clone().downcast::<gtk4::ApplicationWindow>().unwrap();
                                  ui::show_error_dialog(&win, &e);
                              }
                          }
                     } else {
                         // Fallback: Check window content if not in titlebar (e.g. if custom titlebar setup failed)
                         println!("Warning: Could not find 'sync_indicator' in titlebar. Checking window content...");
                         if let Some(child) = window.child() {
                             if let Some(indicator) = find_child_by_name(&child, "sync_indicator") {
                                  println!("Found 'sync_indicator' in window content.");
                                  // Update Class (Optimized duplication...)
                                   indicator.remove_css_class("syncing");
                                   indicator.remove_css_class("synced");
                                   indicator.remove_css_class("error");
                                   match msg {
                                      SyncMsg::Syncing => indicator.add_css_class("syncing"),
                                      SyncMsg::Synced => indicator.add_css_class("synced"),
                                      SyncMsg::Error(_) => indicator.add_css_class("error"),
                                      SyncMsg::LoginSuccess => indicator.set_tooltip_text(Some("Logged In")),
                                      SyncMsg::LoginFailed(e) => {
                                          indicator.add_css_class("error");
                                          let win = window.clone().downcast::<gtk4::ApplicationWindow>().unwrap();
                                          ui::show_error_dialog(&win, &e);
                                      },
                                   }
                             } else {
                                 println!("Error: 'sync_indicator' widget NOT FOUND anywhere.");
                             }
                         }
                     }
                } else {
                    println!("Error: Window has no titlebar!");
                }
            }
        }
        glib::ControlFlow::Continue
    });

    app.run();
}
