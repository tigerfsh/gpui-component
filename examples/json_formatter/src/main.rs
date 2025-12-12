use anyhow::Result;
use gpui::*;
use gpui_component::{button::*, input::*, *};
use serde_json::Value;
use std::io::{self, IsTerminal, Read};

struct JsonFormatter {
    input: Entity<InputState>,
    output: Entity<InputState>,
}

impl JsonFormatter {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .placeholder("Enter JSON here...")
        });
        let output = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .placeholder("Formatted JSON will appear here...")
        });
        Self { input, output }
    }

    fn format(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().to_string();
        let formatted = match serde_json::from_str::<Value>(&text) {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error: {}", e),
        };

        self.output.update(cx, |state, cx| {
            state.set_value(formatted, window, cx);
        });
    }
}

impl Render for JsonFormatter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(div().flex_1().child(Input::new(&self.input).size_full()))
            .child(
                div().flex().justify_center().child(
                    Button::new("format")
                        .primary()
                        .label("Format JSON")
                        .on_click(cx.listener(Self::format)),
                ),
            )
            .child(div().flex_1().child(Input::new(&self.output).size_full()))
    }
}

fn main() -> Result<()> {
    if !std::io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        if buffer.trim().is_empty() {
            return Ok(());
        }

        let v: Value = serde_json::from_str(&buffer)?;
        println!("{}", serde_json::to_string_pretty(&v)?);
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
