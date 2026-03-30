use gpui::{AnyElement, Context, Div, IntoElement, ParentElement, Styled, rgb};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::Input;
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{h_flex, v_flex};

use crate::app::controller::AppCommand;
use crate::core::codecs::{format_hex, reformat_hex, render_payload};
use crate::core::records::{DisplayMode, RecordEntry};

use super::app_view::AppView;
use super::common::{
    BORDER, CHIP_BLUE, CHIP_BLUE_TEXT, CHIP_DARK, PANEL_BG, PANEL_SOFT, TEXT_MAIN, TEXT_MUTED,
    splitter_handle, svg_icon,
};

impl AppView {
    pub(crate) fn render_record_panel(&self) -> AnyElement {
        let mut panel = v_flex()
            .flex_1()
            .overflow_y_scrollbar()
            .p_4()
            .gap_2()
            .bg(rgb(PANEL_BG))
            .border_l_1()
            .border_r_1()
            .border_color(rgb(BORDER));
        if self.state.records.is_empty() && self.state.live_reading.is_none() {
            panel = panel.child(
                v_flex()
                    .flex_1()
                    .min_h(gpui::px(420.))
                    .justify_start()
                    .child(gpui::div()),
            );
        }
        for record in &self.state.records {
            panel = panel.child(render_record_row(record));
        }
        if let Some(live) = &self.state.live_reading {
            panel = panel.child(render_record_row(live));
        }
        panel.into_any_element()
    }

    pub(crate) fn render_control_bar(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let export_weak = weak.clone();
        let read_weak = weak.clone();
        let send_weak = weak.clone();
        let records_clear_weak = weak.clone();
        let send_clear_weak = weak.clone();

        v_flex()
            .w_full()
            .bg(rgb(PANEL_BG))
            .child(
                h_flex()
                    .w_full()
                    .justify_center()
                    .pt_1()
                    .child(splitter_handle()),
            )
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .px_5()
                    .py_3()
                    .border_t_1()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(PANEL_BG))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("read-display")
                                    .outline()
                                    .label(format!(
                                        "⌄ 接收:{}",
                                        self.state.settings.read_display.label()
                                    ))
                                    .text_color(rgb(CHIP_BLUE_TEXT))
                                    .border_color(rgb(CHIP_BLUE_TEXT))
                                    .bg(rgb(CHIP_BLUE))
                                    .h(gpui::px(42.))
                                    .rounded_lg()
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = read_weak.upgrade() {
                                            view.update(cx, |view, ctx| {
                                                view.state.settings.read_display = view
                                                    .state
                                                    .settings
                                                    .read_display
                                                    .next(view.state.settings.has_dec_types);
                                                view.persist_settings();
                                                ctx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new("send-display")
                                    .outline()
                                    .label(format!(
                                        "⌃ 发送:{}",
                                        self.state.settings.send_display.label()
                                    ))
                                    .text_color(rgb(CHIP_BLUE_TEXT))
                                    .border_color(rgb(CHIP_BLUE_TEXT))
                                    .bg(rgb(CHIP_BLUE))
                                    .h(gpui::px(42.))
                                    .rounded_lg()
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = send_weak.upgrade() {
                                            view.update(cx, |view, ctx| {
                                                view.state.settings.send_display = view
                                                    .state
                                                    .settings
                                                    .send_display
                                                    .next(view.state.settings.has_dec_types);
                                                view.persist_settings();
                                                ctx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new("export-session")
                                    .primary()
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .child(svg_icon("clear_context.svg", 14.0))
                                            .child("导出"),
                                    )
                                    .bg(rgb(CHIP_DARK))
                                    .h(gpui::px(42.))
                                    .rounded_lg()
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = export_weak.upgrade() {
                                            view.update(cx, |view, _| {
                                                view.controller.dispatch(AppCommand::ExportCurrent);
                                            });
                                        }
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("clear-records")
                                    .primary()
                                    .label("清空接收")
                                    .bg(rgb(CHIP_DARK))
                                    .h(gpui::px(42.))
                                    .rounded_lg()
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = records_clear_weak.upgrade() {
                                            view.update(cx, |view, ctx| {
                                                view.state.records.clear();
                                                view.state.live_reading = None;
                                                view.state.rx_count = 0;
                                                ctx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new("clear-send")
                                    .primary()
                                    .label("清空发送")
                                    .bg(rgb(CHIP_DARK))
                                    .h(gpui::px(42.))
                                    .rounded_lg()
                                    .on_click(move |_, window, cx| {
                                        if let Some(view) = send_clear_weak.upgrade() {
                                            let input = view.read(cx).send_input.clone();
                                            input.update(cx, |input, ctx| {
                                                input.set_value("", window, ctx);
                                            });
                                        }
                                    }),
                            ),
                    ),
            )
    }

    pub(crate) fn render_send_panel(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let send_weak = weak.clone();
        let auto_weak = weak.clone();
        let add_quick_weak = weak.clone();

        v_flex()
            .w_full()
            .h(gpui::px(240.))
            .bg(rgb(PANEL_BG))
            .child(
                Input::new(&self.send_input)
                    .h_full()
                    .bordered(false)
                    .focus_bordered(false)
                    .bg(rgb(PANEL_BG))
                    .p_3(),
            )
            .child(
                h_flex()
                    .justify_between()
                    .gap_2()
                    .px_5()
                    .py_4()
                    .border_t_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(PANEL_BG))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("checksum")
                                    .ghost()
                                    .h(gpui::px(40.))
                                    .label("校验位"),
                            )
                            .child(
                                Button::new("toggle-auto-send")
                                    .ghost()
                                    .h(gpui::px(40.))
                                    .label(if self.state.is_auto_sending {
                                        "停止自动发送"
                                    } else {
                                        "自动发送"
                                    })
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = auto_weak.upgrade() {
                                            let text = view
                                                .read(cx)
                                                .send_input
                                                .read(cx)
                                                .text()
                                                .to_string();
                                            view.update(cx, |view, _| {
                                                view.toggle_auto_send(text);
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new("add-quick-input")
                                    .ghost()
                                    .h(gpui::px(40.))
                                    .label("加入快捷")
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = add_quick_weak.upgrade() {
                                            let text = view
                                                .read(cx)
                                                .send_input
                                                .read(cx)
                                                .text()
                                                .to_string();
                                            view.update(cx, |view, ctx| {
                                                view.add_quick_input_from_text(text);
                                                ctx.notify();
                                            });
                                        }
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("reformat-hex")
                                    .ghost()
                                    .h(gpui::px(40.))
                                    .label("整理 HEX")
                                    .on_click(move |_, window, cx| {
                                        if let Some(view) = weak.upgrade() {
                                            let input = view.read(cx).send_input.clone();
                                            let is_hex = view.read(cx).state.settings.send_display
                                                == DisplayMode::Hex;
                                            if is_hex {
                                                let formatted = reformat_hex(
                                                    &input.read(cx).text().to_string(),
                                                );
                                                input.update(cx, |input, ctx| {
                                                    input.set_value(formatted, window, ctx);
                                                });
                                            }
                                        }
                                    }),
                            )
                            .child(
                                Button::new("send-current")
                                    .primary()
                                    .label("发送")
                                    .bg(rgb(CHIP_DARK))
                                    .h(gpui::px(40.))
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = send_weak.upgrade() {
                                            let text = view
                                                .read(cx)
                                                .send_input
                                                .read(cx)
                                                .text()
                                                .to_string();
                                            view.update(cx, |view, _| {
                                                view.queue_send_text(text);
                                            });
                                        }
                                    }),
                            ),
                    ),
            )
    }

    pub(crate) fn render_quick_input_panel(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let loop_weak = weak.clone();
        let mut panel = v_flex()
            .w(gpui::px(300.))
            .h_full()
            .p_3()
            .gap_2()
            .border_l_1()
            .border_color(rgb(BORDER))
            .bg(rgb(PANEL_SOFT))
            .child(
                h_flex().justify_between().child("快捷输入").child(
                    Button::new("quick-loop")
                        .ghost()
                        .label(if self.state.is_quick_loop_running {
                            "停止循环"
                        } else {
                            "循环发送"
                        })
                        .on_click(move |_, _, cx| {
                            if let Some(view) = loop_weak.upgrade() {
                                view.update(cx, |view, _| {
                                    view.toggle_quick_loop();
                                });
                            }
                        }),
                ),
            );

        for (index, item) in self.state.quick_inputs.iter().enumerate() {
            let send_item_weak = weak.clone();
            let toggle_item_weak = weak.clone();
            panel = panel.child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .p_2()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(PANEL_BG))
                    .child(
                        v_flex().child(item.content.clone()).child(
                            gpui::div()
                                .text_sm()
                                .text_color(rgb(TEXT_MUTED))
                                .child(format!("{} · {}ms", item.remark, item.delay_ms)),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new(("toggle-quick", index))
                                    .ghost()
                                    .label(if item.enabled {
                                        "已启用"
                                    } else {
                                        "已禁用"
                                    })
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = toggle_item_weak.upgrade() {
                                            view.update(cx, |view, ctx| {
                                                if let Some(item) =
                                                    view.state.quick_inputs.get_mut(index)
                                                {
                                                    item.enabled = !item.enabled;
                                                    view.persist_quick_inputs();
                                                }
                                                ctx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new(("send-quick", index))
                                    .ghost()
                                    .label("发送")
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = send_item_weak.upgrade() {
                                            view.update(cx, |view, _| {
                                                if let Some(item) =
                                                    view.state.quick_inputs.get(index)
                                                {
                                                    view.controller.dispatch(
                                                        AppCommand::SendText {
                                                            text: item.content.clone(),
                                                            display: if item.is_hex {
                                                                DisplayMode::Hex
                                                            } else {
                                                                DisplayMode::Ascii
                                                            },
                                                        },
                                                    );
                                                }
                                            });
                                        }
                                    }),
                            ),
                    ),
            );
        }

        panel
    }

    pub(crate) fn render_status_bar(&self) -> Div {
        h_flex()
            .w_full()
            .h(gpui::px(56.))
            .px_6()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(rgb(BORDER))
            .bg(rgb(PANEL_BG))
            .child(
                h_flex()
                    .gap_3()
                    .text_color(rgb(TEXT_MAIN))
                    .child("⎘")
                    .child("校验位"),
            )
            .child(
                h_flex()
                    .gap_3()
                    .text_color(rgb(TEXT_MAIN))
                    .child("自动发送"),
            )
            .child(
                Button::new("send-final")
                    .primary()
                    .bg(rgb(0x9CA3AF))
                    .w(gpui::px(96.))
                    .h(gpui::px(40.))
                    .label("发送"),
            )
    }
}

pub(super) fn render_record_row(record: &RecordEntry) -> Div {
    let preview = if record.display == DisplayMode::Hex {
        format_hex(&record.data)
    } else {
        render_payload(&record.data, record.display)
    };

    v_flex()
        .w_full()
        .p_3()
        .border_1()
        .border_color(rgb(BORDER))
        .rounded_lg()
        .bg(rgb(0xFCFCFD))
        .child(format!(
            "{} · {} · {}",
            record.kind.label(),
            record.time.format("%H:%M:%S%.3f"),
            record.display.label()
        ))
        .child(
            gpui::div()
                .text_sm()
                .text_color(rgb(TEXT_MAIN))
                .child(preview),
        )
}
