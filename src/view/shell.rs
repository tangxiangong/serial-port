use gpui::{Context, Corner, Div, IntoElement, ParentElement, Styled, rgb};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::Input;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::{Selectable, StyledExt, h_flex, v_flex};

use crate::state::{ConnectionStatus, DeviceType};

use super::app_view::AppView;
use super::common::{
    BORDER, CHIP_DARK, CONTROL_ICON_BG, PANEL_BG, PANEL_SOFT, SIDEBAR_BG, TAB_BG, TEXT_MAIN,
    TEXT_MUTED, svg_icon,
};

impl AppView {
    pub(crate) fn render_activity_bar(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let history_weak = weak.clone();
        let settings_weak = weak.clone();

        v_flex()
            .w(gpui::px(54.))
            .h_full()
            .p_2()
            .gap_5()
            .justify_between()
            .border_r_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SIDEBAR_BG))
            .child(
                v_flex()
                    .items_center()
                    .gap_5()
                    .child(
                        Button::new("github")
                            .ghost()
                            .w(gpui::px(34.))
                            .h(gpui::px(34.))
                            .child(svg_icon("github.svg", 20.0)),
                    )
                    .child(
                        Button::new("bilibili")
                            .ghost()
                            .w(gpui::px(34.))
                            .h(gpui::px(34.))
                            .child(svg_icon("bilibili.svg", 20.0)),
                    ),
            )
            .child(
                v_flex()
                    .items_center()
                    .gap_5()
                    .child(
                        Button::new("history")
                            .ghost()
                            .w(gpui::px(34.))
                            .h(gpui::px(34.))
                            .selected(self.state.history_open)
                            .child(svg_icon("menu.svg", 18.0))
                            .on_click(move |_, _, cx| {
                                if let Some(view) = history_weak.upgrade() {
                                    view.update(cx, |view, cx| {
                                        view.state.history_open = !view.state.history_open;
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("theme")
                            .ghost()
                            .w(gpui::px(34.))
                            .h(gpui::px(34.))
                            .label("◔"),
                    )
                    .child(
                        Button::new("settings")
                            .ghost()
                            .w(gpui::px(34.))
                            .h(gpui::px(34.))
                            .selected(self.state.settings_open)
                            .child(svg_icon("clear_context.svg", 18.0))
                            .on_click(move |_, _, cx| {
                                if let Some(view) = settings_weak.upgrade() {
                                    view.update(cx, |view, cx| {
                                        view.state.settings_open = !view.state.settings_open;
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("more")
                            .ghost()
                            .w(gpui::px(34.))
                            .h(gpui::px(34.))
                            .label("≡"),
                    ),
            )
    }

    pub(crate) fn render_top_bar(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let weak_search = weak.clone();
        let weak_setting = weak.clone();
        let weak_send = weak.clone();
        let weak_quick = weak.clone();

        h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .gap_3()
            .px_6()
            .py_3()
            .border_b_1()
            .border_color(rgb(BORDER))
            .bg(rgb(PANEL_BG))
            .child(gpui::div().w(gpui::px(150.)))
            .child(
                h_flex()
                    .w(gpui::px(360.))
                    .px_4()
                    .py_2()
                    .gap_2()
                    .rounded_lg()
                    .bg(rgb(PANEL_SOFT))
                    .child(gpui::div().text_color(rgb(TEXT_MUTED)).child("⌕"))
                    .child(
                        Input::new(&self.search_input)
                            .w_full()
                            .appearance(false)
                            .bordered(false),
                    )
                    .child(Button::new("search").ghost().label("搜索").on_click(
                        move |_, _, cx| {
                            if let Some(view) = weak_search.upgrade() {
                                let query = view.read(cx).search_input.read(cx).text().to_string();
                                view.update(cx, |view, _| {
                                    view.queue_search(query);
                                });
                            }
                        },
                    )),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("toggle-setting-panel")
                            .outline()
                            .w(gpui::px(36.))
                            .h(gpui::px(36.))
                            .bg(rgb(CONTROL_ICON_BG))
                            .child(svg_icon("menu.svg", 16.0))
                            .on_click(move |_, _, cx| {
                                if let Some(view) = weak_setting.upgrade() {
                                    view.update(cx, |view, ctx| {
                                        view.state.settings.layout.show_setting_panel =
                                            !view.state.settings.layout.show_setting_panel;
                                        view.persist_settings();
                                        ctx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("toggle-send-panel")
                            .outline()
                            .w(gpui::px(36.))
                            .h(gpui::px(36.))
                            .bg(rgb(CONTROL_ICON_BG))
                            .label("◧")
                            .on_click(move |_, _, cx| {
                                if let Some(view) = weak_send.upgrade() {
                                    view.update(cx, |view, ctx| {
                                        view.state.settings.layout.show_send_panel =
                                            !view.state.settings.layout.show_send_panel;
                                        view.persist_settings();
                                        ctx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("toggle-quick-panel")
                            .outline()
                            .w(gpui::px(36.))
                            .h(gpui::px(36.))
                            .bg(rgb(CONTROL_ICON_BG))
                            .label("◨")
                            .on_click(move |_, _, cx| {
                                if let Some(view) = weak_quick.upgrade() {
                                    view.update(cx, |view, ctx| {
                                        view.state.settings.layout.show_quick_input_panel =
                                            !view.state.settings.layout.show_quick_input_panel;
                                        view.persist_settings();
                                        ctx.notify();
                                    });
                                }
                            }),
                    ),
            )
    }

    pub(crate) fn render_device_setting(&self, cx: &mut Context<Self>) -> Div {
        let weak = cx.entity().downgrade();
        let serial_weak = weak.clone();
        let ble_weak = weak.clone();
        let refresh_weak = weak.clone();
        let connect_weak = weak.clone();
        let disconnect_weak = weak.clone();

        let selected_serial = self.state.selected_serial_port.clone();
        let selected_ble = self.state.selected_ble_device.clone();
        let serial_ports = self.state.serial_ports.clone();
        let ble_devices = self.state.ble_devices.clone();
        let baud_rates = self.state.settings.serial.baud_rate_list.clone();
        let current_baud = self.state.settings.serial.baud_rate;
        let current_data_bits = self.state.settings.serial.data_bits;
        let current_parity = self.state.settings.serial.parity.clone();
        let current_stop_bits = self.state.settings.serial.stop_bits;
        let mut panel = v_flex()
            .w(gpui::px(360.))
            .h_full()
            .p_5()
            .gap_5()
            .border_r_1()
            .border_color(rgb(BORDER))
            .bg(rgb(PANEL_SOFT))
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .p_1()
                    .rounded_lg()
                    .bg(rgb(TAB_BG))
                    .child(
                        Button::new("serial-tab")
                            .w_full()
                            .h(gpui::px(44.))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(svg_icon("serial.svg", 16.0))
                                    .child("串口"),
                            )
                            .selected(self.state.settings.device_type == DeviceType::Serial)
                            .on_click(move |_, _, cx| {
                                if let Some(view) = serial_weak.upgrade() {
                                    view.update(cx, |view, ctx| {
                                        view.state.settings.device_type = DeviceType::Serial;
                                        view.persist_settings();
                                        view.refresh_devices();
                                        ctx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("ble-tab")
                            .w_full()
                            .h(gpui::px(44.))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(svg_icon("ble.svg", 16.0))
                                    .child("蓝牙"),
                            )
                            .selected(self.state.settings.device_type == DeviceType::Ble)
                            .on_click(move |_, _, cx| {
                                if let Some(view) = ble_weak.upgrade() {
                                    view.update(cx, |view, ctx| {
                                        view.state.settings.device_type = DeviceType::Ble;
                                        view.persist_settings();
                                        view.refresh_devices();
                                        ctx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .font_semibold()
                            .text_color(rgb(TEXT_MAIN))
                            .child(if self.state.settings.device_type == DeviceType::Serial {
                                "串口设置"
                            } else {
                                "蓝牙设置"
                            }),
                    )
                    .child(gpui::div().text_sm().text_color(rgb(TEXT_MUTED)).child(
                        if self.state.settings.device_type == DeviceType::Serial {
                            "请选择串口连接相关参数"
                        } else {
                            "请选择蓝牙连接相关参数"
                        },
                    )),
            );

        if self.state.settings.device_type == DeviceType::Serial {
            let weak_baud = weak.clone();
            let weak_data_bits = weak.clone();
            let weak_parity = weak.clone();
            let weak_stop_bits = weak.clone();

            panel = panel
                .child(
                    gpui::div()
                        .font_semibold()
                        .text_color(rgb(TEXT_MAIN))
                        .child("波特率"),
                )
                .child(
                    Button::new("baud-rate")
                        .outline()
                        .dropdown_caret(true)
                        .label(current_baud.to_string())
                        .w_full()
                        .h(gpui::px(48.))
                        .bg(rgb(PANEL_BG))
                        .rounded_lg()
                        .dropdown_menu_with_anchor(Corner::TopLeft, move |menu, _, _| {
                            baud_rates.iter().fold(menu, |menu, baud| {
                                let weak_baud = weak_baud.clone();
                                let baud_value = *baud;
                                menu.item(
                                    PopupMenuItem::new(baud.to_string())
                                        .checked(baud_value == current_baud)
                                        .on_click(move |_, _, cx| {
                                            if let Some(view) = weak_baud.upgrade() {
                                                view.update(cx, |view, ctx| {
                                                    view.state.settings.serial.baud_rate =
                                                        baud_value;
                                                    view.persist_settings();
                                                    ctx.notify();
                                                });
                                            }
                                        }),
                                )
                            })
                        }),
                )
                .child(
                    gpui::div()
                        .font_semibold()
                        .text_color(rgb(TEXT_MAIN))
                        .child("数据位"),
                )
                .child(
                    Button::new("data-bits")
                        .outline()
                        .dropdown_caret(true)
                        .label(current_data_bits.to_string())
                        .w_full()
                        .h(gpui::px(48.))
                        .bg(rgb(PANEL_BG))
                        .rounded_lg()
                        .dropdown_menu_with_anchor(Corner::TopLeft, move |menu, _, _| {
                            [8_u8, 7, 6, 5].iter().fold(menu, |menu, value| {
                                let weak_data_bits = weak_data_bits.clone();
                                let bits = *value;
                                menu.item(
                                    PopupMenuItem::new(bits.to_string())
                                        .checked(bits == current_data_bits)
                                        .on_click(move |_, _, cx| {
                                            if let Some(view) = weak_data_bits.upgrade() {
                                                view.update(cx, |view, ctx| {
                                                    view.state.settings.serial.data_bits = bits;
                                                    view.persist_settings();
                                                    ctx.notify();
                                                });
                                            }
                                        }),
                                )
                            })
                        }),
                )
                .child(
                    gpui::div()
                        .font_semibold()
                        .text_color(rgb(TEXT_MAIN))
                        .child("校验位"),
                )
                .child(
                    Button::new("parity")
                        .outline()
                        .dropdown_caret(true)
                        .label(match current_parity.as_str() {
                            "even" => "Even".to_string(),
                            "odd" => "Odd".to_string(),
                            _ => "None".to_string(),
                        })
                        .w_full()
                        .h(gpui::px(48.))
                        .bg(rgb(PANEL_BG))
                        .rounded_lg()
                        .dropdown_menu_with_anchor(Corner::TopLeft, move |menu, _, _| {
                            [("none", "None"), ("even", "Even"), ("odd", "Odd")]
                                .iter()
                                .fold(menu, |menu, (value, label)| {
                                    let weak_parity = weak_parity.clone();
                                    let parity = (*value).to_string();
                                    menu.item(
                                        PopupMenuItem::new(*label)
                                            .checked(current_parity == *value)
                                            .on_click(move |_, _, cx| {
                                                if let Some(view) = weak_parity.upgrade() {
                                                    view.update(cx, |view, ctx| {
                                                        view.state.settings.serial.parity =
                                                            parity.clone();
                                                        view.persist_settings();
                                                        ctx.notify();
                                                    });
                                                }
                                            }),
                                    )
                                })
                        }),
                )
                .child(
                    gpui::div()
                        .font_semibold()
                        .text_color(rgb(TEXT_MAIN))
                        .child("停止位"),
                )
                .child(
                    Button::new("stop-bits")
                        .outline()
                        .dropdown_caret(true)
                        .label(current_stop_bits.to_string())
                        .w_full()
                        .h(gpui::px(48.))
                        .bg(rgb(PANEL_BG))
                        .rounded_lg()
                        .dropdown_menu_with_anchor(Corner::TopLeft, move |menu, _, _| {
                            [1.0_f32, 1.5, 2.0].iter().fold(menu, |menu, value| {
                                let weak_stop_bits = weak_stop_bits.clone();
                                let stop_bits = *value;
                                menu.item(
                                    PopupMenuItem::new(stop_bits.to_string())
                                        .checked((stop_bits - current_stop_bits).abs() < 0.01)
                                        .on_click(move |_, _, cx| {
                                            if let Some(view) = weak_stop_bits.upgrade() {
                                                view.update(cx, |view, ctx| {
                                                    view.state.settings.serial.stop_bits =
                                                        stop_bits;
                                                    view.persist_settings();
                                                    ctx.notify();
                                                });
                                            }
                                        }),
                                )
                            })
                        }),
                );
        }

        panel = panel.child(
            gpui::div()
                .font_semibold()
                .text_color(rgb(TEXT_MAIN))
                .child(if self.state.settings.device_type == DeviceType::Serial {
                    "设备列表"
                } else {
                    "蓝牙设备"
                }),
        );

        panel =
            panel.child(
                gpui::div()
                    .p_3()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .rounded_md()
                    .bg(rgb(PANEL_BG))
                    .child(match self.state.settings.device_type {
                        DeviceType::Serial if self.state.serial_ports.is_empty() => v_flex()
                            .items_center()
                            .gap_2()
                            .child(svg_icon("network-error.svg", 30.0))
                            .child(
                                gpui::div()
                                    .text_sm()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child("未发现串口设备"),
                            )
                            .into_any_element(),
                        DeviceType::Serial => {
                            let label = serial_ports
                                .iter()
                                .find(|port| {
                                    selected_serial.as_deref() == Some(port.port_name.as_str())
                                })
                                .map(|port| format!("{} · {}", port.display_name, port.kind))
                                .unwrap_or_else(|| "请选择串口设备".to_string());
                            let weak_select = weak.clone();
                            Button::new("serial-device-dropdown")
                                .outline()
                                .dropdown_caret(true)
                                .label(label)
                                .w_full()
                                .h(gpui::px(48.))
                                .bg(rgb(PANEL_BG))
                                .rounded_lg()
                                .dropdown_menu_with_anchor(Corner::TopLeft, move |menu, _, _| {
                                    serial_ports.iter().fold(menu, |menu, port| {
                                        let port_name = port.port_name.clone();
                                        let checked = selected_serial.as_deref()
                                            == Some(port.port_name.as_str());
                                        let weak_select = weak_select.clone();
                                        menu.item(
                                            PopupMenuItem::new(format!(
                                                "{} · {}",
                                                port.display_name, port.kind
                                            ))
                                            .checked(checked)
                                            .on_click(move |_, _, cx| {
                                                if let Some(view) = weak_select.upgrade() {
                                                    view.update(cx, |view, ctx| {
                                                        view.state.selected_serial_port =
                                                            Some(port_name.clone());
                                                        ctx.notify();
                                                    });
                                                }
                                            }),
                                        )
                                    })
                                })
                                .into_any_element()
                        }
                        DeviceType::Ble if self.state.ble_devices.is_empty() => v_flex()
                            .items_center()
                            .gap_2()
                            .child(svg_icon("network-error.svg", 30.0))
                            .child(
                                gpui::div()
                                    .text_sm()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child("未发现蓝牙设备"),
                            )
                            .into_any_element(),
                        DeviceType::Ble => {
                            let label = ble_devices
                                .iter()
                                .find(|device| selected_ble.as_deref() == Some(device.id.as_str()))
                                .map(|device| format!("{} · {}", device.name, device.description))
                                .unwrap_or_else(|| "请选择蓝牙设备".to_string());
                            let weak_select = weak.clone();
                            Button::new("ble-device-dropdown")
                                .outline()
                                .dropdown_caret(true)
                                .label(label)
                                .w_full()
                                .h(gpui::px(48.))
                                .bg(rgb(PANEL_BG))
                                .rounded_lg()
                                .dropdown_menu_with_anchor(Corner::TopLeft, move |menu, _, _| {
                                    ble_devices.iter().fold(menu, |menu, device| {
                                        let device_id = device.id.clone();
                                        let checked =
                                            selected_ble.as_deref() == Some(device.id.as_str());
                                        let weak_select = weak_select.clone();
                                        menu.item(
                                            PopupMenuItem::new(format!(
                                                "{} · {}",
                                                device.name, device.description
                                            ))
                                            .checked(checked)
                                            .on_click(move |_, _, cx| {
                                                if let Some(view) = weak_select.upgrade() {
                                                    view.update(cx, |view, ctx| {
                                                        view.state.selected_ble_device =
                                                            Some(device_id.clone());
                                                        ctx.notify();
                                                    });
                                                }
                                            }),
                                        )
                                    })
                                })
                                .into_any_element()
                        }
                    }),
            );

        panel
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("refresh-devices")
                            .outline()
                            .h(gpui::px(48.))
                            .rounded_lg()
                            .bg(rgb(PANEL_BG))
                            .label("刷新")
                            .on_click(move |_, _, cx| {
                                if let Some(view) = refresh_weak.upgrade() {
                                    view.update(cx, |view, _| {
                                        view.refresh_devices();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("connect-device")
                            .primary()
                            .w_full()
                            .h(gpui::px(52.))
                            .rounded_lg()
                            .label(match self.state.connection_status {
                                ConnectionStatus::Connecting => "连接中...",
                                ConnectionStatus::Connected => "已连接",
                                ConnectionStatus::Disconnecting => "断开中...",
                                ConnectionStatus::Idle => {
                                    if self.state.settings.device_type == DeviceType::Serial {
                                        "选择串口设备"
                                    } else {
                                        "选择蓝牙设备"
                                    }
                                }
                            })
                            .bg(rgb(CHIP_DARK))
                            .on_click(move |_, _, cx| {
                                if let Some(view) = connect_weak.upgrade() {
                                    view.update(cx, |view, _| {
                                        view.connect_selected();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("disconnect-device")
                            .danger()
                            .label("断开")
                            .w(gpui::px(84.))
                            .h(gpui::px(52.))
                            .rounded_lg()
                            .on_click(move |_, _, cx| {
                                if let Some(view) = disconnect_weak.upgrade() {
                                    view.update(cx, |view, _| {
                                        view.disconnect();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                gpui::div()
                    .text_sm()
                    .text_color(rgb(TEXT_MUTED))
                    .child(self.state.current_session_label()),
            )
    }
}
