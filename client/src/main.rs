mod db;
mod sync;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use gtk4::glib; // Correct import via gtk4

enum TrayMsg {
    Show,
    Quit,
}

struct TrayHandler {
    sender: glib::Sender<TrayMsg>,
}

impl ksni::Tray for TrayHandler {
    fn icon_name(&self) -> String {
        "accessories-text-editor".into() // Reverted to standard icon
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
                label: "Show Tasks".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.sender.send(TrayMsg::Show);
                }),
                ..Default::default()
            }.into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| {
                     let _ = this.sender.send(TrayMsg::Quit);
                }),
                ..Default::default()
            }.into(),
        ]
    }
}

fn main() {
    let app = Application::builder()
        .application_id("com.ideneyesa.go2do")
        .build();

    // Create a channel to communicate between Tray (thread) and GTK (main loop)
    let (tx, rx) = glib::MainContext::channel(glib::Priority::DEFAULT);
    
    // Start Tray Service
    let service = ksni::TrayService::new(TrayHandler { sender: tx });
    let _tray_handle = service.handle();
    service.spawn();

    // Handle Tray Messages
    let app_clone = app.clone();
    rx.attach(None, move |msg| {
        match msg {
            TrayMsg::Show => {
                let windows = app_clone.windows();
                if let Some(window) = windows.first() {
                    window.unminimize(); // Explicitly unminimize first
                    window.present();   // Then present/raise
                }
            }
            TrayMsg::Quit => {
                std::process::exit(0);
            }
        }
        glib::ControlFlow::Continue
    });

    app.connect_startup(|_| {
        let provider = gtk4::CssProvider::new();
        
        // Try to load from user-local share dir first (install location)
        let mut css_path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        css_path.push(".local/share/go2do/style.css");

        if css_path.exists() {
             provider.load_from_path(css_path.to_str().unwrap());
             println!("Loaded CSS from: {:?}", css_path);
        } else {
             // Fallback to current directory (dev mode)
             provider.load_from_path("style.css");
             println!("Loaded CSS from: style.css (dev)");
        }
        
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    // Start Sync Thread
    println!("Background: Initializing sync with SERVER (https://go2do.ideneyesa.workers.dev)...");
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            loop {
                if let Err(e) = sync::run_sync_cycle().await {
                   eprintln!("Sync Error: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
    });

    app.connect_activate(ui::build_ui);

    app.run();
}
