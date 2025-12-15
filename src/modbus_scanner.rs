use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};
use tokio_modbus::prelude::*;
use tokio_serial::SerialPortBuilderExt;
use tracing::{info, warn};
use crate::configuration::{ModbusConfig as ConfigModbus, ScanConfig};

#[derive(Debug, Clone)]
pub struct ModbusConfig {
    pub device_id: u8,
    pub baud_rate: u32,
    pub parity: tokio_serial::Parity,
    pub stop_bits: tokio_serial::StopBits,
}

#[derive(Debug)]
pub struct ScanResult {
    pub success: bool,
    pub response_data: Option<Vec<u16>>,
    pub response_time_ms: u128,
}

pub struct ModbusScanner {
    usb_device: String,
    modbus_config: ConfigModbus,
    scan_config: ScanConfig,
}

impl ModbusScanner {
    pub fn new(usb_device: String, modbus_config: ConfigModbus, scan_config: ScanConfig) -> Self {
        Self {
            usb_device,
            modbus_config,
            scan_config,
        }
    }

    /// Helper: Convierte string de parity a tokio_serial::Parity
    fn parse_parity(parity_str: &str) -> tokio_serial::Parity {
        match parity_str {
            "N" => tokio_serial::Parity::None,
            "E" => tokio_serial::Parity::Even,
            "O" => tokio_serial::Parity::Odd,
            _ => tokio_serial::Parity::None, // Default
        }
    }

    /// Helper: Convierte u8 a tokio_serial::StopBits
    fn parse_stop_bits(stop_bits: u8) -> tokio_serial::StopBits {
        match stop_bits {
            1 => tokio_serial::StopBits::One,
            2 => tokio_serial::StopBits::Two,
            _ => tokio_serial::StopBits::One, // Default
        }
    }

    pub async fn diagnostic_scan(&self) -> Result<()> {
        info!("🔍 Iniciando barrido rápido de IDs (Función 3: Read Holding Registers)");
        info!("Configuración: {}bps, 8{}{}",
            self.modbus_config.baud_rate,
            self.modbus_config.parity,
            self.modbus_config.stop_bits
        );

        let start_id = self.scan_config.scan_id[0];
        let end_id = self.scan_config.scan_id[1];
        let total_ids = (end_id - start_id + 1) as u64;

        info!("Escaneando IDs: {} - {}", start_id, end_id);
        info!("Registro de prueba: {}", self.scan_config.test_register);

        let bar = ProgressBar::new(total_ids);
        bar.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) - ID: {msg}")
            .unwrap()
            .progress_chars("#=>"));

        let mut found_devices = Vec::new();

        // Parsear configuración de modbus
        let parity = Self::parse_parity(&self.modbus_config.parity);
        let stop_bits = Self::parse_stop_bits(self.modbus_config.stop_bits);

        for device_id in start_id..=end_id {
            bar.set_message(format!("{}", device_id));

            let config = ModbusConfig {
                device_id,
                baud_rate: self.modbus_config.baud_rate,
                parity,
                stop_bits,
            };

            if let Ok(result) = self.test_modbus_config(&config).await {
                if result.success {
                    if let Some(data) = &result.response_data {
                        if !data.is_empty() {
                            bar.println(format!(
                                "✅ ID {} - RESPONDE - Registro {}: {} (Tiempo: {}ms)",
                                device_id,
                                self.scan_config.test_register,
                                data[0],
                                result.response_time_ms
                            ));
                            found_devices.push((device_id, data.clone()));
                        }
                    }
                }
            }

            bar.inc(1);
            tokio::time::sleep(Duration::from_millis(self.scan_config.scan_delay_ms)).await;
        }

        bar.finish_with_message("Escaneo completado");

        // Resumen
        info!("{}", "=".repeat(50));
        info!("Escaneo completado");
        info!("Dispositivos encontrados: {}", found_devices.len());
        if !found_devices.is_empty() {
            info!("IDs que responden:");
            for (id, data) in &found_devices {
                info!("  ID {}: Valor registro {}",  id, data[0]);
            }
        } else {
            warn!("❌ No se encontraron dispositivos");
            warn!("Verifica:");
            warn!("  - Configuración de puerto serie en config/config.toml");
            warn!("  - Conexiones físicas (alimentación, RS485 A/B)");
            warn!("  - Parámetros Modbus (baudrate, parity, stop_bits)");
        }
        info!("{}", "=".repeat(50));

        Ok(())
    }

    /// Prueba una configuración específica
    async fn test_modbus_config(&self, config: &ModbusConfig) -> Result<ScanResult> {
        let start_time = Instant::now();
        
        let builder = tokio_serial::new(&self.usb_device, config.baud_rate)
            .parity(config.parity)
            .stop_bits(config.stop_bits)
            .data_bits(tokio_serial::DataBits::Eight)
            .timeout(Duration::from_millis(self.modbus_config.timeout_ms));
        
        let port_stream = builder
            .open_native_async()
            .context("Error abriendo puerto serie")?;
        
        // Configurar cliente Modbus RTU
        let mut ctx = rtu::attach_slave(port_stream, Slave(config.device_id));

        // Intentar lectura del Holding Register configurado
        match tokio::time::timeout(
            Duration::from_millis(self.modbus_config.timeout_ms),
            ctx.read_holding_registers(self.scan_config.test_register, self.scan_config.register_count)
        ).await {
            Ok(inner_result) => match inner_result {
                Ok(response) => {
                    let response = response?;
                    let elapsed = start_time.elapsed();
                    Ok(ScanResult {
                        success: true,
                        response_data: Some(response),
                        response_time_ms: elapsed.as_millis(),
                    })
                }
                Err(_) => {
                    let elapsed = start_time.elapsed();
                    Ok(ScanResult {
                        success: false,
                        response_data: None,
                        response_time_ms: elapsed.as_millis(),
                    })
                }
            },
            Err(_) => {
                let elapsed = start_time.elapsed();
                Ok(ScanResult {
                    success: false,
                    response_data: None,
                    response_time_ms: elapsed.as_millis(),
                })
            }
        }
    }
}

