use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

/// Data direction for logging.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataDirection {
    Rx,
    Tx,
}

impl DataDirection {
    fn as_str(&self) -> &'static str {
        match self {
            DataDirection::Rx => "RX",
            DataDirection::Tx => "TX",
        }
    }
}

/// Log file format.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Raw,
    Hex,
    Csv,
}

pub struct FileLogger {
    writer: BufWriter<File>,
    format: LogFormat,
}

impl FileLogger {
    pub async fn new(path: PathBuf, format: LogFormat) -> Result<Self> {
        let file = File::create(&path).await?;
        let mut writer = BufWriter::new(file);
        if matches!(format, LogFormat::Csv) {
            writer.write_all(b"timestamp,direction,hex,ascii\n").await?;
        }
        Ok(Self { writer, format })
    }

    pub async fn log_data(&mut self, direction: &DataDirection, data: &[u8]) -> Result<()> {
        match self.format {
            LogFormat::Raw => {
                self.writer.write_all(data).await?;
            }
            LogFormat::Hex => {
                let hex = crate::codec::bytes_to_hex_spaced(data);
                let line = format!(
                    "[{}] {}: {}\n",
                    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    direction.as_str(),
                    hex
                );
                self.writer.write_all(line.as_bytes()).await?;
            }
            LogFormat::Csv => {
                let hex = crate::codec::bytes_to_hex_spaced(data);
                let ascii = crate::codec::bytes_to_ascii_string(data);
                let line = format!(
                    "{},{},{},{}\n",
                    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    direction.as_str(),
                    hex,
                    ascii.replace(',', " ")
                );
                self.writer.write_all(line.as_bytes()).await?;
            }
        }
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn close(mut self) -> Result<()> {
        self.writer.flush().await?;
        Ok(())
    }
}
