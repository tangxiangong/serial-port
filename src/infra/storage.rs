use crate::{
    core::{
        codecs::render_payload,
        records::{QuickInputItem, RecordEntry, SessionMeta},
    },
    state::{AppSettings, BootstrapData, ExportRecord},
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn default_root() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".serial-port-data")
    }

    fn sessions_dir(&self) -> PathBuf {
        self.root.join("sessions")
    }

    fn settings_path(&self) -> PathBuf {
        self.root.join("settings.json")
    }

    fn quick_inputs_path(&self) -> PathBuf {
        self.root.join("quick_inputs.json")
    }

    fn sessions_index_path(&self) -> PathBuf {
        self.sessions_dir().join("index.json")
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.sessions_dir().join(format!("{id}.json"))
    }

    async fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.sessions_dir()).await?;
        Ok(())
    }

    pub async fn bootstrap(&self) -> Result<BootstrapData> {
        self.ensure_dirs().await?;
        Ok(BootstrapData {
            settings: self
                .load_or_default(self.settings_path(), AppSettings::default())
                .await?,
            quick_inputs: self
                .load_or_default(self.quick_inputs_path(), QuickInputItem::default_items())
                .await?,
            sessions: self
                .load_or_default(self.sessions_index_path(), Vec::<SessionMeta>::new())
                .await?,
        })
    }

    pub async fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        self.ensure_dirs().await?;
        self.write_json(self.settings_path(), settings).await
    }

    pub async fn save_quick_inputs(&self, quick_inputs: &[QuickInputItem]) -> Result<()> {
        self.ensure_dirs().await?;
        self.write_json(self.quick_inputs_path(), quick_inputs)
            .await
    }

    pub async fn save_sessions_index(&self, sessions: &[SessionMeta]) -> Result<()> {
        self.ensure_dirs().await?;
        self.write_json(self.sessions_index_path(), sessions).await
    }

    pub async fn save_session_records(
        &self,
        session_id: &str,
        records: &[RecordEntry],
    ) -> Result<()> {
        self.ensure_dirs().await?;
        self.write_json(self.session_path(session_id), records)
            .await
    }

    pub async fn load_session_records(&self, session_id: &str) -> Result<Vec<RecordEntry>> {
        self.load_or_default(self.session_path(session_id), Vec::<RecordEntry>::new())
            .await
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let path = self.session_path(session_id);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(path).await?;
        }
        Ok(())
    }

    pub async fn export_records(&self, records: &[RecordEntry]) -> Result<PathBuf> {
        self.ensure_dirs().await?;
        let exports_dir = self.root.join("exports");
        fs::create_dir_all(&exports_dir).await?;
        let file_name = format!(
            "records-{}.json",
            chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S")
        );
        let path = exports_dir.join(file_name);
        let payload = records
            .iter()
            .map(|record| {
                ExportRecord::from_record(record, render_payload(&record.data, record.display))
            })
            .collect::<Vec<_>>();
        self.write_json(&path, &payload).await?;
        Ok(path)
    }

    async fn write_json<T: serde::Serialize + ?Sized>(
        &self,
        path: impl AsRef<Path>,
        value: &T,
    ) -> Result<()> {
        let data = serde_json::to_vec_pretty(value)?;
        fs::write(path.as_ref(), data)
            .await
            .with_context(|| format!("写入 {} 失败", path.as_ref().display()))
    }

    async fn load_or_default<T>(&self, path: PathBuf, default: T) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(default);
        }
        let bytes = fs::read(&path)
            .await
            .with_context(|| format!("读取 {} 失败", path.display()))?;
        let value = serde_json::from_slice(&bytes)
            .with_context(|| format!("解析 {} 失败", path.display()))?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::SessionStore;
    use crate::core::records::{
        DeviceInfo, DeviceKind, DisplayMode, RecordEntry, RecordKind, SessionMeta,
    };

    #[tokio::test]
    async fn session_roundtrip_works() {
        let root = std::env::temp_dir().join(format!(
            "serial-port-test-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let store = SessionStore::new(root.clone());
        let meta = SessionMeta::new(DeviceInfo {
            kind: DeviceKind::Serial,
            id: "tty.usbmodem".into(),
            name: "USB Serial".into(),
            detail: "USB".into(),
        });
        store
            .save_sessions_index(std::slice::from_ref(&meta))
            .await
            .unwrap();
        let records = vec![RecordEntry::new(
            RecordKind::Write,
            b"AT".to_vec(),
            DisplayMode::Ascii,
        )];
        store
            .save_session_records(&meta.id, &records)
            .await
            .unwrap();
        let loaded = store.load_session_records(&meta.id).await.unwrap();
        assert_eq!(loaded, records);
        let _ = tokio::fs::remove_dir_all(root).await;
    }
}
