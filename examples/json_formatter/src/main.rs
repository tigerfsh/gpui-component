use anyhow::Result;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::InputEvent as ComponentInputEvent;
use gpui_component::{button::*, input::*, *};
use rfd::AsyncFileDialog;
use serde_json::Value;
use std::io::{self, IsTerminal, Read};

struct JsonFormatter {
    input: Entity<InputState>,
    output: Entity<InputState>,
    auto_format: bool,
    compact_mode: bool,
}

impl JsonFormatter {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .placeholder("Enter JSON or JSON5 here...")
        });
        let output = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .placeholder("Formatted JSON will appear here...")
        });

        let view = Self {
            input: input.clone(),
            output,
            auto_format: false,
            compact_mode: false,
        };

        cx.subscribe(&input, move |_, _, event: &ComponentInputEvent, cx| {
            if let ComponentInputEvent::Change = event {
                let weak_view = cx.entity().downgrade();
                cx.spawn(async move |_, cx| {
                    cx.update(|cx| {
                        weak_view
                            .update(cx, |this, cx| {
                                if this.auto_format {
                                    this.format(cx);
                                }
                            })
                            .ok();
                    })
                    .ok();
                })
                .detach();
            }
        })
        .detach();

        view
    }

    fn format(&mut self, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().to_string();
        if text.trim().is_empty() {
            self.output.update(cx, |_state, _cx| {
                // state.set_value("", window, cx);
            });
            return;
        }

        // Try serde_json first, then json5
        let value: Option<Value> = serde_json::from_str(&text)
            .ok()
            .or_else(|| json5::from_str(&text).ok());

        let _formatted = match value {
            Some(v) => {
                if self.compact_mode {
                    serde_json::to_string(&v).unwrap_or_else(|e| e.to_string())
                } else {
                    serde_json::to_string_pretty(&v).unwrap_or_else(|e| e.to_string())
                }
            }
            None => "Invalid JSON or JSON5".to_string(),
        };

        self.output.update(cx, |_state, _cx| {
            // state.set_value(formatted, cx, cx);
        });

        self.output.update(cx, |_state, _cx| {
            // state.set_value(formatted, cx, cx);
        });
    }

    fn on_format_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.format_with_window(window, cx);
    }

    fn format_with_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().to_string();
        if text.trim().is_empty() {
            self.output.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            return;
        }

        let value: Option<Value> = serde_json::from_str(&text)
            .ok()
            .or_else(|| json5::from_str(&text).ok());

        let formatted = match value {
            Some(v) => {
                if self.compact_mode {
                    serde_json::to_string(&v).unwrap_or_else(|e| e.to_string())
                } else {
                    serde_json::to_string_pretty(&v).unwrap_or_else(|e| e.to_string())
                }
            }
            None => "Invalid JSON or JSON5".to_string(),
        };

        self.output.update(cx, |state, cx| {
            state.set_value(formatted, window, cx);
        });
    }

    fn load_file(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let weak_view = cx.entity().downgrade();

        cx.spawn_in(window, async move |_, cx| {
            if let Some(file) = AsyncFileDialog::new().pick_file().await {
                let content = file.read().await;
                let _text = String::from_utf8_lossy(&content).to_string();

                cx.update(|_, cx| {
                    weak_view
                        .update(cx, |this, cx| {
                            this.format(cx);
                        })
                        .ok();
                })
                .ok();
            }
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    }

    fn toggle_compact(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.compact_mode = !self.compact_mode;
        self.format_with_window(window, cx);
    }

    fn clear(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.input
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.output
            .update(cx, |state, cx| state.set_value("", window, cx));
    }

    fn toggle_auto_format(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.auto_format = !self.auto_format;
        if self.auto_format {
            self.format_with_window(window, cx);
        }
    }
}

impl Render for JsonFormatter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(gpui::white())
            .child(
                // Title Bar / Toolbar
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .p_2()
                    .border_b_1()
                    .border_color(gpui::rgb(0xcccccc))
                    .child(
                        Button::new("load")
                            .outline()
                            .label("Load File")
                            .on_click(cx.listener(Self::load_file)),
                    )
                    .child(
                        Button::new("compact")
                            .outline()
                            .label(if self.compact_mode {
                                "Expand"
                            } else {
                                "Compress"
                            })
                            .on_click(cx.listener(Self::toggle_compact)),
                    )
                    .child(
                        Button::new("clear")
                            .outline()
                            .label("Clear")
                            .on_click(cx.listener(Self::clear)),
                    )
                    .child(
                        Button::new("auto_format")
                            .outline()
                            .label(if self.auto_format {
                                "Auto Format: ON"
                            } else {
                                "Auto Format: OFF"
                            })
                            .on_click(cx.listener(Self::toggle_auto_format)),
                    ),
            )
            .child(
                // Main Content
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .p_4()
                    .child(div().flex_1().child(Input::new(&self.input).size_full()))
                    .when(!self.auto_format, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .justify_center()
                                .items_center()
                                .child(
                                    Button::new("format_btn")
                                        // .primary()
                                        .small()
                                        .label(">>>")
                                        .on_click(cx.listener(Self::on_format_click)),
                                ),
                        )
                    })
                    .child(div().flex_1().child(Input::new(&self.output).size_full())),
            )
    }
}

fn main() -> Result<()> {
    if !std::io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        if buffer.trim().is_empty() {
            return Ok(());
        }

        // Try serde_json first, then json5
        let value: Option<Value> = serde_json::from_str(&buffer)
            .ok()
            .or_else(|| json5::from_str(&buffer).ok());

        match value {
            Some(v) => println!("{}", serde_json::to_string_pretty(&v)?),
            None => eprintln!("Invalid JSON or JSON5"),
        }
        return Ok(());
    }

    let app = Application::new();
    app.run(move |cx| {
        gpui_component::init(cx);
        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| JsonFormatter::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
    Ok(())
}
