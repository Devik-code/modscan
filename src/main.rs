use anyhow::Result;
use tracing::info;

mod configuration;
mod logger;
mod modbus_scanner;

use configuration::Configuration;
use modbus_scanner::ModbusScanner;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Configuration::load()?;
    let _guard = logger::logger_init(&config.log_dir_dev, &config.log_dir_prod);

    info!("🚀 Iniciando scanner Modbus");

    let scanner = ModbusScanner::new(config.usb_device.clone(), config.modbus, config.scan);
    scanner.diagnostic_scan().await?;

    Ok(())
}
