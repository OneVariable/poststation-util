use std::sync::atomic::{AtomicU32, Ordering};

use embedded_hal_async::i2c::{Error, ErrorType, I2c, Operation};
use i2c_passthru_icd::{
    self, I2cError, I2cReadEndpoint, I2cWriteEndpoint, I2cWriteReadEndpoint, ReadCommand,
    WriteCommand, WriteReadCommand,
};
use poststation_sdk::{connect, SquadClient};

struct I2cDev {
    serial: u64,
    client: SquadClient,
    ctr: AtomicU32,
}

#[derive(Debug)]
enum HostI2CError {
    ConnectionError,
    DeviceError,
    NotYetSupported,
}

impl Error for HostI2CError {
    fn kind(&self) -> embedded_hal_async::i2c::ErrorKind {
        embedded_hal_async::i2c::ErrorKind::Other
    }
}

impl ErrorType for I2cDev {
    type Error = HostI2CError;
}

impl I2c for I2cDev {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        // TODO: impl operations for real. For now, it'd probably be easy to just
        // support read/write/write_then_read.
        match operations {
            [] => Ok(()),
            [Operation::Read(buf)] => self.read(address, buf).await,
            [Operation::Write(buf)] => self.write(address, buf).await,
            [Operation::Write(tx), Operation::Read(rx)] => self.write_read(address, tx, rx).await,
            _ => Err(HostI2CError::NotYetSupported),
        }
    }

    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let Ok(res) = self
            .client
            .proxy_endpoint::<I2cReadEndpoint>(
                self.serial,
                self.ctr(),
                &ReadCommand {
                    addr: address,
                    len: read.len() as u32,
                },
            )
            .await
        else {
            return Err(HostI2CError::ConnectionError);
        };

        let Ok(data) = res else {
            return Err(HostI2CError::DeviceError);
        };

        read.copy_from_slice(&data.data);
        Ok(())
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let res = self
            .client
            .proxy_endpoint::<I2cWriteEndpoint>(
                self.serial,
                self.ctr(),
                &WriteCommand {
                    addr: address,
                    data: write.to_vec(),
                },
            )
            .await;

        match res {
            Ok(Ok(())) => Ok(()),
            Ok(Err(I2cError)) => Err(HostI2CError::DeviceError),
            Err(_) => Err(HostI2CError::ConnectionError),
        }
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let res = self
            .client
            .proxy_endpoint::<I2cWriteReadEndpoint>(
                self.serial,
                self.ctr(),
                &WriteReadCommand {
                    addr: address,
                    tx_data: write.to_vec(),
                    rx_len: read.len() as u32,
                },
            )
            .await;

        match res {
            Ok(Ok(resp)) => {
                read.copy_from_slice(&resp.data);
                Ok(())
            }
            Ok(Err(I2cError)) => Err(HostI2CError::DeviceError),
            Err(_) => Err(HostI2CError::ConnectionError),
        }
    }
}

impl I2cDev {
    pub fn new(client: SquadClient, serial: u64) -> Self {
        Self {
            serial,
            client,
            ctr: AtomicU32::new(0),
        }
    }

    #[inline(always)]
    fn ctr(&self) -> u32 {
        self.ctr.fetch_add(1, Ordering::Relaxed)
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    const SERIAL: u64 = 0xE66350865F164926;
    let client = connect("localhost:51837").await;
    let mut i2c = I2cDev::new(client, SERIAL);

    // Use our client device as if it was a local I2C port with
    // embedded-hal-async traits
    let mut data = [0u8; 4];
    let addr = 0x42;
    i2c.read(addr, &mut data).await.unwrap();
    println!("{data:02X?}");

    Ok(())
}
