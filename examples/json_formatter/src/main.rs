//! A JSON formatter tool with GPUI UI

use gpui::*;
use gpui_component::{
    h_flex, v_flex,
    button::Button,
    label::Label,
    input::{Input, InputEvent, InputState},
    Root, *
};
use gpui_component_assets::Assets;
use serde_json::Value;
use std::fs;
use tracing::{info, Level};
use tracing_subscriber;

actions!(
    json_formatter,
    [
        OpenFile,
        ToggleCompression,
        Clear,
        OpenSettings,
    ]
);

pub struct JsonFormatter {
    input_editor: Entity<InputState>,
    output_editor: Entity<InputState>,
    error_message: Option<SharedString>,
    compression_enabled: bool,
    _subscriptions: Vec<Subscription>,
}

impl JsonFormatter {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_editor = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .code_editor("json")
                .line_number(true)
                .placeholder("Enter JSON here...")
        });
        
        let output_editor = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .code_editor("json")
                .line_number(true)
                .placeholder("Formatted JSON will appear here...")
        });

        let _subscriptions = vec![
            cx.subscribe_in(&input_editor, window, {
                move |this, _, ev: &InputEvent, window, cx| match ev {
                    InputEvent::Change => {
                        this.parse_input(window, cx);
                        cx.notify();
                    }
                    _ => {}
                }
            })
        ];

        Self {
            input_editor,
            output_editor,
            error_message: None,
            compression_enabled: false,
            _subscriptions,
        }
    }

    fn parse_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        info!("Parsing input text");
        
        let input_text = self.input_editor.read(cx).value();
        if input_text.is_empty() {
            self.output_editor.update(cx, |state, cx| {
                state.set_value("".to_string(), window, cx);
            });
            self.error_message = None;
            return;
        }

        // Try parsing with serde_json first
        match serde_json::from_str::<Value>(&input_text) {
            Ok(value) => {
                info!("Parsed successfully with serde_json");
                self.format_output(value, window, cx);
                self.error_message = None;
            }
            Err(serde_err) => {
                // If serde_json fails, try with json5
                match json5::from_str::<Value>(&input_text) {
                    Ok(value) => {
                        info!("Parsed successfully with json5");
                        self.format_output(value, window, cx);
                        self.error_message = None;
                    }
                    Err(json5_err) => {
                        // Both parsers failed, show error
                        info!("Failed to parse with both serde_json and json5");
                        self.error_message = Some(format!(
                            "JSON parsing error:\nserde_json: {}\njson5: {}",
                            serde_err, json5_err
                        ).into());
                        
                        self.output_editor.update(cx, |state, cx| {
                            state.set_value("".to_string(), window, cx);
                        });
                    }
                }
            }
        }
    }

    fn format_output(&mut self, value: Value, window: &mut Window, cx: &mut Context<Self>) {
        let formatted = if self.compression_enabled {
            // Compress to single line
            match serde_json::to_string(&value) {
                Ok(s) => s,
                Err(e) => {
                    self.error_message = Some(format!("Formatting error: {}", e).into());
                    return;
                }
            }
        } else {
            // Pretty print with indentation
            match serde_json::to_string_pretty(&value) {
                Ok(s) => s,
                Err(e) => {
                    self.error_message = Some(format!("Formatting error: {}", e).into());
                    return;
                }
            }
        };

        self.output_editor.update(cx, |state, cx| {
            state.set_value(formatted, window, cx);
        });
    }

    fn open_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        info!("Opening file dialog");
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select a JSON file".into()),
        });

        let view = cx.entity();
        cx.spawn_in(window, async move |_, window| {
            if let Ok(Ok(Some(paths))) = prompt.await {
                if let Some(path) = paths.first() {
                    match fs::read_to_string(path) {
                        Ok(content) => {
                            _ = window.update(|window, cx| {
                                _ = view.update(cx, |view: &mut JsonFormatter, cx| {
                                    view.input_editor.update(cx, |state: &mut InputState, cx| {
                                        state.set_value(content, window, cx);
                                    });
                                    view.parse_input(window, cx);
                                });
                            });
                        }
                        Err(e) => {
                            _ = window.update(|window, cx| {
                                _ = view.update(cx, |view: &mut JsonFormatter, cx| {
                                    view.error_message = Some(format!("Error reading file: {}", e).into());
                                    cx.notify();
                                });
                            });
                        }
                    }
                }
            }
        })
        .detach();
    }

    fn toggle_compression(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.compression_enabled = !self.compression_enabled;
        info!("Toggled compression: {}", self.compression_enabled);
        // Re-parse to apply new formatting
        self.parse_input(window, cx);
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        info!("Clearing input and output");
        self.input_editor.update(cx, |state: &mut InputState, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.output_editor.update(cx, |state: &mut InputState, cx| {
            state.set_value("".to_string(), window, cx);
        });
        self.error_message = None;
        cx.notify();
    }

    fn open_settings(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        info!("Opening settings");
        cx.notify();
    }
}

impl Render for JsonFormatter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.bind_keys(vec![
            KeyBinding::new("cmd-o", OpenFile, None),
            KeyBinding::new("cmd-e", ToggleCompression, None),
            KeyBinding::new("cmd-k", Clear, None),
        ]);
        
        v_flex()
            .size_full()
            .child(self.render_menu_bar(cx))
            .child(
                h_flex()
                    .size_full()
                    .child(self.render_input_panel(cx))
                    .child(self.render_output_panel(cx)),
            )
            .child(self.render_error_panel())
    }
}

impl JsonFormatter {
    fn render_menu_bar(&self, cx: &Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .h_10()
            .bg(cx.theme().secondary)
            .border_b_1()
            .border_color(cx.theme().border)
            .items_center()
            .px_2()
            .gap_2()
            .child(
                Button::new("open-btn")
                    .label("Open File")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_file(window, cx);
                    })),
            )
            .child(
                Button::new("compress-btn")
                    .label(if self.compression_enabled {
                        "Expand"
                    } else {
                        "Compress"
                    })
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_compression(window, cx);
                    })),
            )
            .child(
                Button::new("clear-btn")
                    .label("Clear")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.clear(window, cx);
                    })),
            )
            .child(
                Button::new("settings-btn")
                    .label("Settings")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_settings(window, cx);
                    })),
            )
    }

    fn render_input_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_1_2()
            .h_full()
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .h_8()
                    .px_2()
                    .items_center()
                    .child(Label::new("Input")),
            )
            .child(
                Input::new(&self.input_editor)
                    .h_full()
                    .w_full()
            )
    }

    fn render_output_panel(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_1_2()
            .h_full()
            .child(
                h_flex()
                    .h_8()
                    .px_2()
                    .items_center()
                    .child(Label::new("Output")),
            )
            .child(
                Input::new(&self.output_editor)
                    .h_full()
                    .w_full()
            )
    }

    fn render_error_panel(&self) -> impl IntoElement {
        if let Some(error) = &self.error_message {
            v_flex()
                .w_full()
                .h_24()
                .bg(rgb(0xff3333))
                .text_color(gpui::white())
                .p_2()
                .child(Label::new(error.clone()))
        } else {
            v_flex().w_full().h_24()
        }
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting JSON Formatter application");

    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        info!("Initializing components");
        gpui_component::init(cx);
        cx.activate(true);
        
        info!("Setting up window");
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("JSON Formatter".into()),
                appears_transparent: false,
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(options, |window, cx| {
                let view = cx.new(|cx| JsonFormatter::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;
            
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}