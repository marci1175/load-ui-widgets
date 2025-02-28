use std::sync::{Arc, Mutex};

use egui::{Response, Ui};
use mlua::Lua;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Application {
    #[serde(skip)]
    ui_elements: Arc<Mutex<Vec<Box<dyn Fn(&mut Ui)-> Response + Send + Sync>>>>,

    #[serde(skip)]
    lua_engine: Lua,
}

impl Default for Application {
    fn default() -> Self {
        let ui_elements = Arc::new(Mutex::new(vec![]));
        Self {
            lua_engine: setup_lua_engine(ui_elements.clone()),
            ui_elements,
        }
    }
}

impl Application {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

fn setup_lua_engine(ui_elements: Arc<Mutex<Vec<Box<dyn Fn(&mut Ui) -> Response + Send + Sync>>>>) -> Lua {
    let lua_engine = unsafe { Lua::unsafe_new() };

    let ui_elements = ui_elements.clone();

    let function = lua_engine.create_function(move |_, label: String| {
        let mut ui_elements = ui_elements.lock().unwrap();

        ui_elements.push(Box::new(move |ui| {ui.label(label.clone())}));

        Ok(())
    }).unwrap();

    lua_engine.globals().set("ui_label", function).unwrap();

    lua_engine
}

impl eframe::App for Application {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Load").clicked() {
                if let Err(err) = self.lua_engine.load(include_str!("../test.lua")).exec() {
                    dbg!(err);
                };
            }

            for widget in self.ui_elements.lock().unwrap().iter() {
                ui.add(widget);
            }
        });
    }
}