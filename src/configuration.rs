use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ModbusConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: String,
    pub stop_bits: u8,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScanConfig {
    pub scan_id: Vec<u8>,
    pub test_register: u16,
    pub register_count: u16,
    pub scan_delay_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub log_dir_dev: String,
    pub log_dir_prod: String,
    pub usb_device: String,
    pub modbus: ModbusConfig,
    pub scan: ScanConfig,
}

impl Configuration {
    pub fn load() -> Result<Configuration> {
        // Intentar múltiples ubicaciones para el archivo de configuración
        let config_paths = vec![
            "/var/lib/modscan/config.toml",  // Ubicación de instalación
            "config/config.toml",             // Ubicación de desarrollo
        ];

        let mut config_file = None;
        for path in &config_paths {
            if std::path::Path::new(path).exists() {
                config_file = Some(*path);
                break;
            }
        }

        let config_path = config_file.context("NO configuration file found! Tried: /var/lib/modscan/config.toml, config/config.toml")?;

        let cfg = Config::builder()
            .add_source(File::with_name(config_path))
            .build()
            .context("Failed to load configuration file")?;

        let config: Configuration = cfg
            .try_deserialize()
            .context("Failed to deserialize configuration - check TOML format")?;

        // use print instead of info (log), because to init log we need the path from here.
        println!("{config:#?}");

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.usb_device.is_empty() {
            anyhow::bail!("usb_device cannot be empty");
        }

        // Validate Modbus config

        if self.modbus.data_bits != 7 && self.modbus.data_bits != 8 {
            anyhow::bail!("modbus.data_bits must be 7 or 8");
        }
        if self.modbus.parity != "N" && self.modbus.parity != "E" && self.modbus.parity != "O" {
            anyhow::bail!("Invalid parity value: {}. Must be 'N', 'E', or 'O'", self.modbus.parity);
        }
        if self.modbus.stop_bits != 1 && self.modbus.stop_bits != 2 {
            anyhow::bail!("stop_bits must be 1 or 2, got {}", self.modbus.stop_bits);
        }

        // Validate Scan config
        if self.scan.scan_id.len() != 2 {
            anyhow::bail!("scan.scan_id must contain exactly two elements: [start_id, end_id]");
        }
        let start_id = self.scan.scan_id[0];
        let end_id = self.scan.scan_id[1];

        if start_id == 0 {
            anyhow::bail!("scan.scan_id[0] (start_id) must be >= 1");
        }
        if end_id > 247 {
            anyhow::bail!("scan.scan_id[1] (end_id) must be <= 247");
        }
        if start_id > end_id {
            anyhow::bail!("scan.scan_id[0] (start_id) must be <= scan.scan_id[1] (end_id)");
        }

        Ok(())
    }
}