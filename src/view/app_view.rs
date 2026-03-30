use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Task,
    Window, rgb,
};
use gpui_component::input::InputState;
use gpui_component::{ActiveTheme, h_flex, v_flex};

use crate::app::controller::{AppCommand, AppController, AppEvent};
use crate::core::records::{DisplayMode, QuickInputItem};
use crate::state::{AppState, DeviceType};

use super::common::{APP_BG, PANEL_BG};

pub struct AppView {
    pub(super) controller: AppController,
    pub(super) state: AppState,
    pub(super) send_input: Entity<InputState>,
    pub(super) search_input: Entity<InputState>,
    pub(super) _subscriptions: Vec<Subscription>,
    pub(super) _event_task: Task<()>,
}

impl AppView {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        controller: AppController,
        mut event_rx: tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    ) -> Self {
        let send_input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .rows(6)
                .placeholder("输入要发送的数据")
        });
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("搜索记录"));

        let event_task = cx.spawn(async move |this, cx| {
            while let Some(event) = event_rx.recv().await {
                if let Some(view) = this.upgrade() {
                    let _ = view.update(cx, |view, cx| {
                        view.apply_event(event, cx);
                    });
                } else {
                    break;
                }
            }
        });

        controller.dispatch(AppCommand::Bootstrap);

        Self {
            controller,
            state: AppState::default(),
            send_input,
            search_input,
            _subscriptions: Vec::new(),
            _event_task: event_task,
        }
    }

    pub(super) fn apply_event(&mut self, event: AppEvent, _cx: &mut Context<Self>) {
        match event {
            AppEvent::Bootstrapped(data) => {
                self.state.bootstrapped = true;
                self.state.settings = data.settings;
                self.state.quick_inputs = data.quick_inputs;
                self.state.sessions = data.sessions;
            }
            AppEvent::SerialPortsUpdated(ports) => {
                self.state.serial_ports = ports;
                if self
                    .state
                    .selected_serial_port
                    .as_ref()
                    .is_none_or(|selected| {
                        !self
                            .state
                            .serial_ports
                            .iter()
                            .any(|port| &port.port_name == selected)
                    })
                {
                    self.state.selected_serial_port = self
                        .state
                        .serial_ports
                        .first()
                        .map(|port| port.port_name.clone());
                }
            }
            AppEvent::BleDevicesUpdated(devices) => {
                self.state.ble_devices = devices;
                if self
                    .state
                    .selected_ble_device
                    .as_ref()
                    .is_none_or(|selected| {
                        !self
                            .state
                            .ble_devices
                            .iter()
                            .any(|device| &device.id == selected)
                    })
                {
                    self.state.selected_ble_device = self
                        .state
                        .ble_devices
                        .first()
                        .map(|device| device.id.clone());
                }
            }
            AppEvent::ConnectionStatusChanged(status) => self.state.connection_status = status,
            AppEvent::ActiveDeviceChanged(device) => self.state.active_device = device,
            AppEvent::RecordCommitted(record) => {
                self.state.finalize_live_read();
                self.state.apply_record(record);
            }
            AppEvent::LiveReadChunk(display, chunk) => self.state.begin_live_read(display, chunk),
            AppEvent::LiveReadFinished => {
                self.state.finalize_live_read();
            }
            AppEvent::SearchResults(results) => {
                self.state.search_results = results;
                self.state.search_open = true;
            }
            AppEvent::SessionsUpdated(sessions) => self.state.sessions = sessions,
            AppEvent::SessionLoaded(id, records) => {
                self.state.selected_session_id = Some(id);
                self.state.selected_session_records = records;
                self.state.history_open = true;
            }
            AppEvent::SessionExported(path) => {
                self.state.latest_notice = format!("已导出到 {path}")
            }
            AppEvent::AutoSendState(running) => self.state.is_auto_sending = running,
            AppEvent::QuickLoopState(running) => self.state.is_quick_loop_running = running,
            AppEvent::Notice(message) => self.state.latest_notice = message,
            AppEvent::Error(message) => {
                self.state.error_notice = Some(message.clone());
                self.state.latest_notice = message;
            }
        }
    }

    pub(super) fn persist_settings(&self) {
        self.controller
            .dispatch(AppCommand::PersistSettings(self.state.settings.clone()));
    }

    pub(super) fn persist_quick_inputs(&self) {
        self.controller.dispatch(AppCommand::PersistQuickInputs(
            self.state.quick_inputs.clone(),
        ));
    }

    pub(super) fn refresh_devices(&self) {
        match self.state.settings.device_type {
            DeviceType::Serial => self.controller.dispatch(AppCommand::RefreshSerialPorts),
            DeviceType::Ble => self.controller.dispatch(AppCommand::RefreshBleDevices),
        }
    }

    pub(super) fn connect_selected(&mut self) {
        match self.state.settings.device_type {
            DeviceType::Serial => {
                if let Some(port_name) = &self.state.selected_serial_port {
                    self.controller.dispatch(AppCommand::ConnectSerial {
                        port_name: port_name.clone(),
                        config: self.state.settings.serial.clone(),
                    });
                } else {
                    self.state.latest_notice = "没有可连接的串口设备".into();
                }
            }
            DeviceType::Ble => {
                if let Some(device_id) = &self.state.selected_ble_device {
                    self.controller.dispatch(AppCommand::ConnectBle {
                        device_id: device_id.clone(),
                    });
                } else {
                    self.state.latest_notice = "没有可连接的蓝牙设备".into();
                }
            }
        }
    }

    pub(super) fn disconnect(&self) {
        self.controller.dispatch(AppCommand::Disconnect);
    }

    pub(super) fn queue_send_text(&self, text: String) {
        self.controller.dispatch(AppCommand::SendText {
            text,
            display: self.state.settings.send_display,
        });
    }

    pub(super) fn queue_search(&self, query: String) {
        self.controller.dispatch(AppCommand::Search(query));
    }

    pub(super) fn toggle_auto_send(&self, text: String) {
        if self.state.is_auto_sending {
            self.controller.dispatch(AppCommand::StopAutoSend);
        } else {
            self.controller.dispatch(AppCommand::StartAutoSend {
                text,
                display: self.state.settings.send_display,
                interval_ms: self.state.settings.auto_send_interval_ms,
            });
        }
    }

    pub(super) fn toggle_quick_loop(&self) {
        if self.state.is_quick_loop_running {
            self.controller.dispatch(AppCommand::StopQuickLoop);
        } else {
            self.controller.dispatch(AppCommand::StartQuickLoop {
                items: self.state.quick_inputs.clone(),
                interval_ms: self.state.settings.quick_loop_interval_ms,
            });
        }
    }

    pub(super) fn add_quick_input_from_text(&mut self, content: String) {
        if content.trim().is_empty() {
            self.state.latest_notice = "发送框为空，无法加入快捷输入".into();
            return;
        }
        self.state.quick_inputs.push(QuickInputItem {
            id: format!("quick-{}", self.state.quick_inputs.len() + 1),
            enabled: true,
            content,
            remark: "从发送框添加".into(),
            delay_ms: 1_000,
            is_hex: self.state.settings.send_display == DisplayMode::Hex,
        });
        self.persist_quick_inputs();
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut content = h_flex()
            .size_full()
            .bg(rgb(APP_BG))
            .child(self.render_activity_bar(cx))
            .child({
                let mut main = v_flex()
                    .flex_1()
                    .size_full()
                    .bg(rgb(APP_BG))
                    .child(self.render_top_bar(cx))
                    .child({
                        let mut work = h_flex().flex_1();
                        if self.state.settings.layout.show_setting_panel {
                            work = work.child(self.render_device_setting(cx));
                        }
                        let mut center = v_flex().flex_1().h_full().bg(rgb(PANEL_BG));
                        center = center.child(self.render_record_panel());
                        let mut bottom = v_flex().w_full().flex_none();
                        bottom = bottom.child(self.render_control_bar(cx));
                        if self.state.settings.layout.show_send_panel {
                            bottom = bottom.child(self.render_send_panel(cx));
                        }
                        bottom = bottom.child(self.render_status_bar());
                        center = center.child(bottom);
                        work = work.child(center);
                        if self.state.settings.layout.show_quick_input_panel {
                            work = work.child(self.render_quick_input_panel(cx));
                        }
                        work
                    });
                if self.state.history_open {
                    main = main.child(self.render_history_drawer(cx));
                }
                if self.state.search_open {
                    main = main.child(self.render_search_overlay(cx));
                }
                if self.state.settings_open {
                    main = main.child(self.render_settings_overlay(cx));
                }
                main
            });

        if let Some(error) = &self.state.error_notice {
            content = content.child(
                gpui::div()
                    .absolute()
                    .left(gpui::px(90.))
                    .bottom(gpui::px(36.))
                    .w(gpui::px(420.))
                    .p_2()
                    .border_1()
                    .bg(cx.theme().danger)
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        content
    }
}
