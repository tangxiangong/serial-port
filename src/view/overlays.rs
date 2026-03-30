use gpui::{Context, Div, ParentElement, Styled};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{ActiveTheme, h_flex, v_flex};

use super::app_view::AppView;
use super::workspace::render_record_row;

impl AppView {
    pub(crate) fn render_history_drawer(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let close_weak = weak.clone();
        let mut sessions_column = v_flex().gap_2().overflow_y_scrollbar().flex_1();

        for (index, session) in self.state.sessions.iter().enumerate() {
            let session_id = session.id.clone();
            let delete_session_id = session.id.clone();
            let load_weak = weak.clone();
            let delete_weak = weak.clone();
            sessions_column = sessions_column.child(
                v_flex()
                    .p_2()
                    .border_1()
                    .child(session.title.clone())
                    .child(gpui::div().text_sm().child(format!(
                        "{} · {} 条记录",
                        session.device.name, session.record_count
                    )))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new(("load-session", index))
                                    .ghost()
                                    .label("查看")
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = load_weak.upgrade() {
                                            view.update(cx, |view, _| {
                                                view.controller.dispatch(crate::app::controller::AppCommand::LoadSession(
                                                    session_id.clone(),
                                                ));
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new(("delete-session", index))
                                    .danger()
                                    .label("删除")
                                    .on_click(move |_, _, cx| {
                                        if let Some(view) = delete_weak.upgrade() {
                                            view.update(cx, |view, _| {
                                                view.controller.dispatch(
                                                    crate::app::controller::AppCommand::DeleteSession(
                                                        delete_session_id.clone(),
                                                    ),
                                                );
                                            });
                                        }
                                    }),
                            ),
                    ),
            );
        }

        let mut records_column = v_flex().gap_2().overflow_y_scrollbar().flex_1();
        for record in &self.state.selected_session_records {
            records_column = records_column.child(render_record_row(record));
        }

        gpui::div()
            .absolute()
            .top(gpui::px(0.))
            .right(gpui::px(0.))
            .bottom(gpui::px(0.))
            .w(gpui::px(420.))
            .border_l_1()
            .bg(cx.theme().background)
            .child(
                v_flex()
                    .size_full()
                    .p_3()
                    .gap_3()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child("会话历史")
                            .child(Button::new("close-history").ghost().label("关闭").on_click(
                                move |_, _, cx| {
                                    if let Some(view) = close_weak.upgrade() {
                                        view.update(cx, |view, ctx| {
                                            view.state.history_open = false;
                                            ctx.notify();
                                        });
                                    }
                                },
                            )),
                    )
                    .child(
                        gpui::div()
                            .text_sm()
                            .child(format!("{} 个会话", self.state.sessions.len())),
                    )
                    .child(sessions_column)
                    .child(gpui::div().border_t_1())
                    .child(records_column),
            )
    }

    pub(crate) fn render_search_overlay(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let close_weak = weak.clone();
        let mut content = v_flex().gap_2();
        for result in &self.state.search_results {
            content = content.child(
                v_flex()
                    .w_full()
                    .p_2()
                    .border_1()
                    .child(format!("{} · {}", result.kind, result.timestamp))
                    .child(gpui::div().text_sm().child(result.content.clone())),
            );
        }

        gpui::div()
            .absolute()
            .top(gpui::px(56.))
            .left(gpui::px(100.))
            .w(gpui::px(560.))
            .max_h(gpui::px(360.))
            .border_1()
            .bg(cx.theme().background)
            .shadow_lg()
            .child(
                v_flex()
                    .size_full()
                    .p_3()
                    .gap_2()
                    .child(h_flex().justify_between().child("搜索结果").child(
                        Button::new("close-search").ghost().label("关闭").on_click(
                            move |_, _, cx| {
                                if let Some(view) = close_weak.upgrade() {
                                    view.update(cx, |view, ctx| {
                                        view.state.search_open = false;
                                        ctx.notify();
                                    });
                                }
                            },
                        ),
                    ))
                    .child(gpui::div().overflow_y_scrollbar().child(content)),
            )
    }

    pub(crate) fn render_settings_overlay(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let close_weak = weak.clone();
        let dec_weak = weak.clone();

        gpui::div()
            .absolute()
            .top(gpui::px(80.))
            .left(gpui::px(180.))
            .w(gpui::px(480.))
            .border_1()
            .bg(cx.theme().background)
            .shadow_lg()
            .child(
                v_flex()
                    .p_4()
                    .gap_3()
                    .child(
                        h_flex().justify_between().child("设置").child(
                            Button::new("close-settings")
                                .ghost()
                                .label("关闭")
                                .on_click(move |_, _, cx| {
                                    if let Some(view) = close_weak.upgrade() {
                                        view.update(cx, |view, ctx| {
                                            view.state.settings_open = false;
                                            ctx.notify();
                                        });
                                    }
                                }),
                        ),
                    )
                    .child(
                        h_flex().justify_between().child("十进制模式").child(
                            Button::new("toggle-dec")
                                .ghost()
                                .label(if self.state.settings.has_dec_types {
                                    "已启用"
                                } else {
                                    "已禁用"
                                })
                                .on_click(move |_, _, cx| {
                                    if let Some(view) = dec_weak.upgrade() {
                                        view.update(cx, |view, ctx| {
                                            view.state.settings.has_dec_types =
                                                !view.state.settings.has_dec_types;
                                            view.persist_settings();
                                            ctx.notify();
                                        });
                                    }
                                }),
                        ),
                    )
                    .child(gpui::div().child(format!(
                        "自动发送间隔: {} ms · 快捷循环间隔: {} ms",
                        self.state.settings.auto_send_interval_ms,
                        self.state.settings.quick_loop_interval_ms
                    ))),
            )
    }
}
