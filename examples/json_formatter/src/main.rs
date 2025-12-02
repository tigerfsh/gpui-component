use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputEvent, InputState},
    *,
};
use gpui_component_assets::Assets;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq)]
enum OutputMode {
    Formatted,
    Minified,
}

#[derive(Clone, Copy, PartialEq)]
enum Language {
    English,
    Chinese,
}


pub struct JsonFormatter {
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
    last_format_time: Option<Instant>,
    output_mode: OutputMode,
    language: Language,
    error_message: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl JsonFormatter {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("json")
                .multi_line(true)
                .line_number(true)
                .indent_guides(true)
                .soft_wrap(true)
                .default_value(r#"{"name": "example", "items": [1, 2, 3]}"#)
                .placeholder("Paste JSON here or load from file...")
        });

        let output_state = cx.new(|cx| {
            InputState::new(window, cx)
                .auto_grow(10, 30)
                .placeholder("Formatted JSON will appear here...")
        });

        let subscriptions = vec![cx.subscribe_in(&input_state, window, {
            move |this, _, _: &InputEvent, window, cx| {
                this.format_json(window, cx);
            }
        })];

        let mut formatter = Self {
            input_state,
            output_state,
            last_format_time: None,
            output_mode: OutputMode::Formatted,
            language: Language::English,
            error_message: None,
            _subscriptions: subscriptions,
        };

        // Initial format
        formatter.format_json(window, cx);

        formatter
    }

    fn format_json(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.input_state.read(cx).value();
        if input.trim().is_empty() {
            self.output_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            self.error_message = None;
            cx.notify();
            return;
        }

        match json5::from_str::<serde_json::Value>(&input) {
            Ok(value) => {
                let output = match self.output_mode {
                    OutputMode::Formatted => serde_json::to_string_pretty(&value).unwrap_or_default(),
                    OutputMode::Minified => serde_json::to_string(&value).unwrap_or_default(),
                };
                self.output_state.update(cx, |state, cx| {
                    state.set_value(&output, window, cx);
                });
                self.error_message = None;
                self.last_format_time = Some(Instant::now());
            }
            Err(e) => {
                self.output_state.update(cx, |state, cx| {
                    state.set_value("", window, cx);
                });
                self.error_message = Some(e.to_string());
            }
        }

        cx.notify();
    }

    fn load_from_file(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let path = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select a JSON file".into()),
        });

        let input_state = self.input_state.downgrade();
        cx.spawn_in(window, async move |_, window| {
            match path.await {
                Ok(inner) => match inner {
                    Ok(Some(mut paths)) => {
                        let path = paths.remove(0);
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                input_state.update_in(window, |state, window, cx| {
                                    state.set_value(&content, window, cx);
                                }).ok();
                            }
                            Err(e) => {
                                // Could show error dialog here
                                println!("Error reading file: {}", e);
                            }
                        }
                    }
                    Ok(None) => {
                        // User cancelled
                    }
                    Err(e) => {
                        println!("Error selecting file: {}", e);
                    }
                },
                Err(_) => {
                    // Cancelled
                }
            }

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    }

    fn copy_to_clipboard(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let output = self.output_state.read(cx).value();
        if !output.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(output.to_string()));
        }
    }

    fn clear(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        self.output_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        self.error_message = None;
        cx.notify();
    }

    fn toggle_format(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        // Toggle output mode
        self.output_mode = match self.output_mode {
            OutputMode::Formatted => OutputMode::Minified,
            OutputMode::Minified => OutputMode::Formatted,
        };
        // Regenerate output with new mode
        self.format_json(window, cx);
    }

    fn toggle_language(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.language = match self.language {
            Language::English => Language::Chinese,
            Language::Chinese => Language::English,
        };
        cx.notify();
    }

    fn translate(&self, key: &'static str) -> &'static str {
        match self.language {
            Language::English => match key {
                "load_file" => "Load File",
                "copy" => "Copy",
                "clear" => "Clear",
                "minify" => "Minify",
                "format" => "Format",
                "language" => "English",
                "status_formatted" => "JSON formatted successfully",
                "status_minified" => "JSON minified successfully",
                "error_prefix" => "Error: ",
                _ => key,
            },
            Language::Chinese => match key {
                "load_file" => "加载文件",
                "copy" => "复制",
                "clear" => "清空",
                "minify" => "压缩",
                "format" => "展开",
                "language" => "中文",
                "status_formatted" => "JSON 格式化成功",
                "status_minified" => "JSON 压缩成功",
                "error_prefix" => "错误: ",
                _ => key,
            },
        }
    }
}

impl Render for JsonFormatter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status_text = if let Some(error) = &self.error_message {
            format!("{}{}", self.translate("error_prefix"), error)
        } else if self.output_mode == OutputMode::Formatted {
            self.translate("status_formatted").to_string()
        } else {
            self.translate("status_minified").to_string()
        };

        v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("load-file")
                                    .label(self.translate("load_file"))
                                    .on_click(cx.listener(Self::load_from_file))
                            )
                            .child(
                                Button::new("copy")
                                    .label(self.translate("copy"))
                                    .on_click(cx.listener(Self::copy_to_clipboard))
                            )
                            .child(
                                Button::new("clear")
                                    .label(self.translate("clear"))
                                    .on_click(cx.listener(Self::clear))
                            )
                            .child(
                                Button::new("toggle-format")
                                    .label(if self.output_mode == OutputMode::Formatted {
                                        self.translate("minify")
                                    } else {
                                        self.translate("format")
                                    })
                                    .on_click(cx.listener(Self::toggle_format))
                            )
                    )
                    .child(
                        Button::new("language")
                            .label(self.translate("language"))
                            .on_click(cx.listener(Self::toggle_language))
                    )
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_1()
                    .h_full()
                    .child(Input::new(&self.input_state).h_full())
                    .child(Input::new(&self.output_state).h_full().disabled(true))
            )
            .child(
                div()
                    .text_xs()
                    .text_color(if self.error_message.is_some() {
                        cx.theme().danger_foreground
                    } else {
                        cx.theme().muted_foreground
                    })
                    .child(status_text)
            )
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| JsonFormatter::new(window, cx));
                // This first level on the window, should be a Root.
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}