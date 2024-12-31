use std::sync::atomic::{AtomicU32, Ordering};

use i2c_passthru_icd;
use poststation_sdk::{connect, SquadClient};

struct I2cDev {
    serial: u64,
    client: SquadClient,
    ctr: AtomicU32,
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
    let i2c = I2cDev::new(client, SERIAL);

    Ok(())
}
