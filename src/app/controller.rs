use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::app::search_service::SearchService;
use crate::core::codecs::encode_payload;
use crate::core::records::{
    DeviceInfo, DisplayMode, QuickInputItem, RecordEntry, RecordKind, SessionMeta,
};
use crate::core::search::SearchResult;
use crate::infra::storage::SessionStore;
use crate::infra::transport::{self, ActiveTransport, TransportEvent};
use crate::state::{
    AppSettings, BleDeviceDescriptor, BootstrapData, ConnectionStatus, SerialConfig,
    SerialPortDescriptor,
};

#[derive(Debug, Clone)]
pub enum AppCommand {
    Bootstrap,
    RefreshSerialPorts,
    RefreshBleDevices,
    ConnectSerial {
        port_name: String,
        config: SerialConfig,
    },
    ConnectBle {
        device_id: String,
    },
    Disconnect,
    SendText {
        text: String,
        display: DisplayMode,
    },
    StartAutoSend {
        text: String,
        display: DisplayMode,
        interval_ms: u64,
    },
    StopAutoSend,
    StartQuickLoop {
        items: Vec<QuickInputItem>,
        interval_ms: u64,
    },
    StopQuickLoop,
    Search(String),
    PersistSettings(AppSettings),
    PersistQuickInputs(Vec<QuickInputItem>),
    LoadSession(String),
    DeleteSession(String),
    ExportCurrent,
    InternalTransport(TransportEvent),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Bootstrapped(BootstrapData),
    SerialPortsUpdated(Vec<SerialPortDescriptor>),
    BleDevicesUpdated(Vec<BleDeviceDescriptor>),
    ConnectionStatusChanged(ConnectionStatus),
    ActiveDeviceChanged(Option<DeviceInfo>),
    RecordCommitted(RecordEntry),
    LiveReadChunk(DisplayMode, Vec<u8>),
    LiveReadFinished,
    SearchResults(Vec<SearchResult>),
    SessionsUpdated(Vec<SessionMeta>),
    SessionLoaded(String, Vec<RecordEntry>),
    SessionExported(String),
    AutoSendState(bool),
    QuickLoopState(bool),
    Notice(String),
    Error(String),
}

#[derive(Clone)]
pub struct AppController {
    tx: mpsc::UnboundedSender<AppCommand>,
}

impl AppController {
    pub fn new(runtime: Arc<Handle>) -> (Self, mpsc::UnboundedReceiver<AppEvent>) {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let worker = AppWorker::new(
            command_tx.clone(),
            event_tx,
            SessionStore::new(SessionStore::default_root()),
        );
        runtime.spawn(worker.run(command_rx));
        (Self { tx: command_tx }, event_rx)
    }

    pub fn dispatch(&self, command: AppCommand) {
        let _ = self.tx.send(command);
    }
}

struct CurrentSession {
    meta: SessionMeta,
    records: Vec<RecordEntry>,
}

struct AppWorker {
    command_tx: mpsc::UnboundedSender<AppCommand>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    store: SessionStore,
    settings: AppSettings,
    quick_inputs: Vec<QuickInputItem>,
    search: SearchService,
    sessions: Vec<SessionMeta>,
    current_session: Option<CurrentSession>,
    active_transport: Option<ActiveTransport>,
    flush_task: Option<JoinHandle<()>>,
    auto_send_token: Option<CancellationToken>,
    quick_loop_token: Option<CancellationToken>,
}

impl AppWorker {
    fn new(
        command_tx: mpsc::UnboundedSender<AppCommand>,
        event_tx: mpsc::UnboundedSender<AppEvent>,
        store: SessionStore,
    ) -> Self {
        Self {
            command_tx,
            event_tx,
            store,
            settings: AppSettings::default(),
            quick_inputs: QuickInputItem::default_items(),
            search: SearchService::default(),
            sessions: Vec::new(),
            current_session: None,
            active_transport: None,
            flush_task: None,
            auto_send_token: None,
            quick_loop_token: None,
        }
    }

    async fn run(mut self, mut rx: mpsc::UnboundedReceiver<AppCommand>) {
        let _ = self.handle(AppCommand::Bootstrap).await;
        while let Some(command) = rx.recv().await {
            if let Err(error) = self.handle(command).await {
                let _ = self.event_tx.send(AppEvent::Error(error.to_string()));
            }
        }
    }

    async fn handle(&mut self, command: AppCommand) -> Result<()> {
        match command {
            AppCommand::Bootstrap => self.bootstrap().await,
            AppCommand::RefreshSerialPorts => self.refresh_serial_ports().await,
            AppCommand::RefreshBleDevices => self.refresh_ble_devices().await,
            AppCommand::ConnectSerial { port_name, config } => {
                self.connect_serial(&port_name, &config).await
            }
            AppCommand::ConnectBle { device_id } => self.connect_ble(&device_id).await,
            AppCommand::Disconnect => self.disconnect().await,
            AppCommand::SendText { text, display } => self.send_text(text, display).await,
            AppCommand::StartAutoSend {
                text,
                display,
                interval_ms,
            } => self.start_auto_send(text, display, interval_ms).await,
            AppCommand::StopAutoSend => {
                self.stop_auto_send();
                Ok(())
            }
            AppCommand::StartQuickLoop { items, interval_ms } => {
                self.start_quick_loop(items, interval_ms).await
            }
            AppCommand::StopQuickLoop => {
                self.stop_quick_loop();
                Ok(())
            }
            AppCommand::Search(query) => self.search(query).await,
            AppCommand::PersistSettings(settings) => {
                self.settings = settings.clone();
                self.store.save_settings(&settings).await
            }
            AppCommand::PersistQuickInputs(quick_inputs) => {
                self.quick_inputs = quick_inputs.clone();
                self.store.save_quick_inputs(&quick_inputs).await
            }
            AppCommand::LoadSession(id) => self.load_session(&id).await,
            AppCommand::DeleteSession(id) => self.delete_session(&id).await,
            AppCommand::ExportCurrent => self.export_current().await,
            AppCommand::InternalTransport(event) => self.handle_transport_event(event).await,
        }
    }

    async fn bootstrap(&mut self) -> Result<()> {
        let data = self.store.bootstrap().await?;
        self.settings = data.settings.clone();
        self.quick_inputs = data.quick_inputs.clone();
        self.sessions = data.sessions.clone();
        let _ = self.event_tx.send(AppEvent::Bootstrapped(data));
        let _ = self
            .event_tx
            .send(AppEvent::Notice("已加载本地状态".into()));
        let _ = self.command_tx.send(AppCommand::RefreshSerialPorts);
        let _ = self.command_tx.send(AppCommand::RefreshBleDevices);
        Ok(())
    }

    async fn refresh_serial_ports(&mut self) -> Result<()> {
        let ports = transport::serial::list_ports().await?;
        let _ = self.event_tx.send(AppEvent::SerialPortsUpdated(ports));
        Ok(())
    }

    async fn refresh_ble_devices(&mut self) -> Result<()> {
        let profile = self.settings.active_ble_profile();
        let devices = transport::ble::list_devices(&profile)
            .await
            .unwrap_or_default();
        let _ = self.event_tx.send(AppEvent::BleDevicesUpdated(devices));
        Ok(())
    }

    async fn connect_serial(&mut self, port_name: &str, config: &SerialConfig) -> Result<()> {
        self.disconnect().await?;
        let _ = self.event_tx.send(AppEvent::ConnectionStatusChanged(
            ConnectionStatus::Connecting,
        ));
        let mut transport =
            ActiveTransport::Serial(transport::serial::connect(port_name, config).await?);
        self.attach_transport(&mut transport).await?;
        self.active_transport = Some(transport);
        Ok(())
    }

    async fn connect_ble(&mut self, device_id: &str) -> Result<()> {
        self.disconnect().await?;
        let _ = self.event_tx.send(AppEvent::ConnectionStatusChanged(
            ConnectionStatus::Connecting,
        ));
        let mut transport = ActiveTransport::Ble(
            transport::ble::connect(device_id, &self.settings.active_ble_profile()).await?,
        );
        self.attach_transport(&mut transport).await?;
        self.active_transport = Some(transport);
        Ok(())
    }

    async fn attach_transport(&mut self, transport: &mut ActiveTransport) -> Result<()> {
        let device = transport.device();
        let mut stream = transport.take_events();
        let command_tx = self.command_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = stream.next().await {
                let _ = command_tx.send(AppCommand::InternalTransport(event));
            }
        });
        if let Some(device) = device {
            self.start_session(device).await?;
        }
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.stop_auto_send();
        self.stop_quick_loop();
        if let Some(mut transport) = self.active_transport.take() {
            let _ = self.event_tx.send(AppEvent::ConnectionStatusChanged(
                ConnectionStatus::Disconnecting,
            ));
            transport.disconnect().await?;
        }
        let _ = self.event_tx.send(AppEvent::ActiveDeviceChanged(None));
        let _ = self
            .event_tx
            .send(AppEvent::ConnectionStatusChanged(ConnectionStatus::Idle));
        Ok(())
    }

    async fn start_session(&mut self, device: DeviceInfo) -> Result<()> {
        let meta = SessionMeta::new(device.clone());
        self.sessions.insert(0, meta.clone());
        self.store.save_sessions_index(&self.sessions).await?;
        self.current_session = Some(CurrentSession {
            meta,
            records: Vec::new(),
        });
        let _ = self
            .event_tx
            .send(AppEvent::ActiveDeviceChanged(Some(device)));
        let _ = self.event_tx.send(AppEvent::ConnectionStatusChanged(
            ConnectionStatus::Connected,
        ));
        let _ = self
            .event_tx
            .send(AppEvent::SessionsUpdated(self.sessions.clone()));
        Ok(())
    }

    async fn send_text(&mut self, text: String, display: DisplayMode) -> Result<()> {
        let bytes = encode_payload(
            &text,
            display,
            &self.settings.line_ending(),
            if display == DisplayMode::Hex {
                self.settings.selected_checksum
            } else {
                None
            },
        )?;
        let Some(transport) = &self.active_transport else {
            let _ = self
                .event_tx
                .send(AppEvent::Error("当前没有已连接设备".into()));
            return Ok(());
        };
        transport.send(&bytes).await?;
        let record = RecordEntry::new(RecordKind::Write, bytes, display);
        self.commit_record(record).await?;
        Ok(())
    }

    async fn commit_record(&mut self, record: RecordEntry) -> Result<()> {
        if let Some(session) = &mut self.current_session {
            session.meta.record_count += 1;
            session.meta.updated_at = chrono::Utc::now();
            session.records.push(record.clone());
            if let Some(meta) = self
                .sessions
                .iter_mut()
                .find(|meta| meta.id == session.meta.id)
            {
                *meta = session.meta.clone();
            }
            self.store.save_sessions_index(&self.sessions).await?;
            let session_id = session.meta.id.clone();
            let records = session.records.clone();
            let store = self.store.clone();
            if let Some(task) = self.flush_task.take() {
                task.abort();
            }
            self.flush_task = Some(tokio::spawn(async move {
                sleep(Duration::from_millis(500)).await;
                let _ = store.save_session_records(&session_id, &records).await;
            }));
        }
        self.search
            .append(
                self.current_records_len().saturating_sub(1),
                std::slice::from_ref(&record),
            )
            .await;
        let _ = self.event_tx.send(AppEvent::RecordCommitted(record));
        let _ = self
            .event_tx
            .send(AppEvent::SessionsUpdated(self.sessions.clone()));
        Ok(())
    }

    fn current_records_len(&self) -> usize {
        self.current_session
            .as_ref()
            .map(|session| session.records.len())
            .unwrap_or(0)
    }

    async fn handle_transport_event(&mut self, event: TransportEvent) -> Result<()> {
        match event {
            TransportEvent::Connected(device) => {
                let _ = self
                    .event_tx
                    .send(AppEvent::ActiveDeviceChanged(Some(device)));
                let _ = self.event_tx.send(AppEvent::ConnectionStatusChanged(
                    ConnectionStatus::Connected,
                ));
            }
            TransportEvent::Disconnected => {
                self.stop_auto_send();
                self.stop_quick_loop();
                let _ = self
                    .event_tx
                    .send(AppEvent::ConnectionStatusChanged(ConnectionStatus::Idle));
                let _ = self.event_tx.send(AppEvent::Notice("设备已断开".into()));
            }
            TransportEvent::BytesReceived(chunk) => {
                let _ = self
                    .event_tx
                    .send(AppEvent::LiveReadChunk(self.settings.read_display, chunk));
            }
            TransportEvent::FrameReceived(frame) => {
                let record = RecordEntry::new(RecordKind::Read, frame, self.settings.read_display);
                self.commit_record(record).await?;
                let _ = self.event_tx.send(AppEvent::LiveReadFinished);
            }
            TransportEvent::Error(message) => {
                let _ = self.event_tx.send(AppEvent::Error(message));
            }
        }
        Ok(())
    }

    async fn search(&mut self, query: String) -> Result<()> {
        let results = self.search.query(&query).await;
        let _ = self.event_tx.send(AppEvent::SearchResults(results));
        Ok(())
    }

    async fn load_session(&mut self, id: &str) -> Result<()> {
        let records = self.store.load_session_records(id).await?;
        self.search.rebuild(&records).await;
        let _ = self
            .event_tx
            .send(AppEvent::SessionLoaded(id.to_string(), records));
        Ok(())
    }

    async fn delete_session(&mut self, id: &str) -> Result<()> {
        self.sessions.retain(|item| item.id != id);
        self.store.delete_session(id).await?;
        self.store.save_sessions_index(&self.sessions).await?;
        let _ = self
            .event_tx
            .send(AppEvent::SessionsUpdated(self.sessions.clone()));
        Ok(())
    }

    async fn export_current(&mut self) -> Result<()> {
        let Some(session) = &self.current_session else {
            let _ = self
                .event_tx
                .send(AppEvent::Error("当前没有活动会话".into()));
            return Ok(());
        };
        let path = self.store.export_records(&session.records).await?;
        let _ = self
            .event_tx
            .send(AppEvent::SessionExported(path.display().to_string()));
        Ok(())
    }

    async fn start_auto_send(
        &mut self,
        text: String,
        display: DisplayMode,
        interval_ms: u64,
    ) -> Result<()> {
        self.stop_auto_send();
        let token = CancellationToken::new();
        let command_tx = self.command_tx.clone();
        let loop_token = token.clone();
        tokio::spawn(async move {
            loop {
                if loop_token.is_cancelled() {
                    break;
                }
                let _ = command_tx.send(AppCommand::SendText {
                    text: text.clone(),
                    display,
                });
                tokio::select! {
                    _ = loop_token.cancelled() => break,
                    _ = sleep(Duration::from_millis(interval_ms.max(10))) => {}
                }
            }
        });
        self.auto_send_token = Some(token);
        let _ = self.event_tx.send(AppEvent::AutoSendState(true));
        Ok(())
    }

    fn stop_auto_send(&mut self) {
        if let Some(token) = self.auto_send_token.take() {
            token.cancel();
        }
        let _ = self.event_tx.send(AppEvent::AutoSendState(false));
    }

    async fn start_quick_loop(
        &mut self,
        items: Vec<QuickInputItem>,
        interval_ms: u64,
    ) -> Result<()> {
        self.stop_quick_loop();
        let token = CancellationToken::new();
        let command_tx = self.command_tx.clone();
        let loop_token = token.clone();
        tokio::spawn(async move {
            let enabled = items
                .into_iter()
                .filter(|item| item.enabled)
                .collect::<Vec<_>>();
            if enabled.is_empty() {
                return;
            }
            loop {
                if loop_token.is_cancelled() {
                    break;
                }
                for item in &enabled {
                    let _ = command_tx.send(AppCommand::SendText {
                        text: item.content.clone(),
                        display: if item.is_hex {
                            DisplayMode::Hex
                        } else {
                            DisplayMode::Ascii
                        },
                    });
                    tokio::select! {
                        _ = loop_token.cancelled() => return,
                        _ = sleep(Duration::from_millis(item.delay_ms.max(10))) => {}
                    }
                }
                tokio::select! {
                    _ = loop_token.cancelled() => break,
                    _ = sleep(Duration::from_millis(interval_ms.max(50))) => {}
                }
            }
        });
        self.quick_loop_token = Some(token);
        let _ = self.event_tx.send(AppEvent::QuickLoopState(true));
        Ok(())
    }

    fn stop_quick_loop(&mut self) {
        if let Some(token) = self.quick_loop_token.take() {
            token.cancel();
        }
        let _ = self.event_tx.send(AppEvent::QuickLoopState(false));
    }
}
