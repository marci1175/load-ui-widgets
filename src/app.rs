use std::sync::{atomic::AtomicBool, Arc, Mutex};

use egui::{Response, Ui};
use mlua::{AnyUserData, FromLuaMulti, Function, IntoLua, IntoLuaMulti, Lua, LuaSerdeExt, MultiValue, Table, UserData};

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

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TextEditOutput {
    pub buffer: Arc<Mutex<String>>,
}

impl UserData for TextEditOutput {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_buffer", |_, this, _: ()| {
            Ok(this.buffer.lock().unwrap().clone())
        });
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct CheckBoxOutput {
    pub is_checked: Arc<Mutex<bool>>,
}

impl UserData for CheckBoxOutput {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_buffer", |_, this, _: ()| {
            Ok(this.is_checked.lock().unwrap().clone())
        });
    }
}

fn setup_lua_engine(ui_elements: Arc<Mutex<Vec<Box<dyn Fn(&mut Ui) -> Response + Send + Sync>>>>) -> Lua {
    let lua_engine = unsafe { Lua::unsafe_new() };

    let ui_elements_clone = ui_elements.clone();

    let ui_label = lua_engine.create_function(move |_, label: String| {
        let mut ui_elements = ui_elements_clone.lock().unwrap();

        ui_elements.push(Box::new(move |ui| {ui.label(label.clone())}));

        Ok(())
    }).unwrap();

    lua_engine.globals().set("ui_label", ui_label).unwrap();

    let ui_elements_clone = ui_elements.clone();

    let ui_textedit = lua_engine.create_function(move |_, _: ()| {
        let mut ui_elements = ui_elements_clone.lock().unwrap();

        let buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

        let buffer_clone = buffer.clone();

        ui_elements.push(Box::new(move |ui| {ui.text_edit_singleline(&mut *buffer_clone.lock().unwrap())}));

        Ok(TextEditOutput { buffer })
    }).unwrap();

    lua_engine.globals().set("ui_textedit", ui_textedit).unwrap();

    let ui_elements_clone = ui_elements.clone();

    let ui_button = lua_engine.create_function(move |_, (label, callback): (String, Function)| {
        let mut ui_elements = ui_elements_clone.lock().unwrap();

        ui_elements.push(Box::new(move |ui| {
            let button = ui.button(label.clone());

            if button.clicked() {
                callback.call::<()>(()).unwrap();
            }

            button
        }));

        Ok(())
    }).unwrap();

    lua_engine.globals().set("ui_button", ui_button).unwrap();

    let ui_elements_clone = ui_elements.clone();

    let ui_separator = lua_engine.create_function(move |_, _: ()| {
        let mut ui_elements = ui_elements_clone.lock().unwrap();

        ui_elements.push(Box::new(move |ui| {
            ui.separator()
        }));

        Ok(())
    }).unwrap();

    lua_engine.globals().set("ui_separator", ui_separator).unwrap();

    let ui_elements_clone = ui_elements.clone();

    let ui_checkbox = lua_engine.create_function(move |_, label: String| {
        let mut ui_elements = ui_elements_clone.lock().unwrap();

        let is_checked = Arc::new(Mutex::new(false));
        
        let is_checked_clone = is_checked.clone();

        ui_elements.push(Box::new(move |ui| {
            ui.checkbox(&mut is_checked_clone.lock().unwrap(), label.clone())
        }));

        Ok(CheckBoxOutput { is_checked })
    }).unwrap();

    lua_engine.globals().set("ui_checkbox", ui_checkbox).unwrap();

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

            if let Ok(draw_fn) = self.lua_engine.globals().get::<Function>("on_draw") {
                let _: () = draw_fn.call(()).unwrap();
            }


            for widget in self.ui_elements.lock().unwrap().iter() {
                ui.add(widget);
            }
        });
    }
}