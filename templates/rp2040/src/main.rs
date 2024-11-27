#![no_std]
#![no_main]

use app::AppTx;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, gpio::{Level, Output}, peripherals::USB, usb};
use embassy_time::{Duration, Instant, Ticker};
use embassy_usb::{Config, UsbDevice};
use postcard_rpc::{sender_fmt, server::{Dispatch, Sender, Server}};
use static_cell::StaticCell;

bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

use {defmt_rtt as _, panic_probe as _};

pub mod app;
pub mod handlers;


fn usb_config(serial: &'static str) -> Config<'static> {
    let mut config = Config::new(0x16c0, 0x27DD);
    config.manufacturer = Some("OneVariable");
    config.product = Some("poststation-pico");
    config.serial_number = Some(serial);

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // SYSTEM INIT
    info!("Start");
    let mut p = embassy_rp::init(Default::default());
    // Obtain the flash ID
    let unique_id = unique_id::get_unique_id(&mut p.FLASH).unwrap();
    static SERIAL_STRING: StaticCell<[u8; 16]> = StaticCell::new();
    let mut ser_buf = [b' '; 16];
    // This is a simple number-to-hex formatting
    unique_id
        .to_be_bytes()
        .iter()
        .zip(ser_buf.chunks_exact_mut(2))
        .for_each(|(b, chs)| {
            let mut b = *b;
            for c in chs {
                *c = match b >> 4 {
                    v @ 0..10 => b'0' + v,
                    v @ 10..16 => b'A' + (v - 10),
                    _ => b'X',
                };
                b <<= 4;
            }
        });
    let ser_buf = SERIAL_STRING.init(ser_buf);
    let ser_buf = core::str::from_utf8(ser_buf.as_slice()).unwrap();

    // USB/RPC INIT
    let driver = usb::Driver::new(p.USB, Irqs);
    let pbufs = app::PBUFS.take();
    let config = usb_config(ser_buf);
    let led = Output::new(p.PIN_25, Level::Low);

    let context = app::Context { unique_id, led };

    let (device, tx_impl, rx_impl) = app::STORAGE.init_poststation(driver, config, pbufs.tx_buf.as_mut_slice());
    let dispatcher = app::MyApp::new(context, spawner.into());
    let vkk = dispatcher.min_key_len();
    let mut server: app::AppServer = Server::new(
        tx_impl,
        rx_impl,
        pbufs.rx_buf.as_mut_slice(),
        dispatcher,
        vkk,
    );
    let sender = server.sender();
    // We need to spawn the USB task so that USB messages are handled by
    // embassy-usb
    spawner.must_spawn(usb_task(device));
    spawner.must_spawn(logging_task(sender));

    // Begin running!
    loop {
        // If the host disconnects, we'll return an error here.
        // If this happens, just wait until the host reconnects
        let _ = server.run().await;
    }
}

/// This handles the low level USB management
#[embassy_executor::task]
pub async fn usb_task(mut usb: UsbDevice<'static, app::AppDriver>) {
    usb.run().await;
}

/// This task is a "sign of life" logger
#[embassy_executor::task]
pub async fn logging_task(sender: Sender<AppTx>) {
    let mut ticker = Ticker::every(Duration::from_secs(3));
    let start = Instant::now();
    loop {
        ticker.next().await;
        let _ = sender_fmt!(sender, "Uptime: {:?}", start.elapsed()).await;
    }
}

/// Helper to get unique ID from flash
mod unique_id {
    use embassy_rp::{
        flash::{Blocking, Flash},
        peripherals::FLASH,
    };

    /// This function retrieves the unique ID of the external flash memory.
    ///
    /// The RP2040 has no internal unique ID register, but most flash chips do,
    /// So we use that instead.
    pub fn get_unique_id(flash: &mut FLASH) -> Option<u64> {
        let mut flash: Flash<'_, FLASH, Blocking, { 16 * 1024 * 1024 }> =
            Flash::new_blocking(flash);
        let mut id = [0u8; core::mem::size_of::<u64>()];
        flash.blocking_unique_id(&mut id).ok()?;
        Some(u64::from_be_bytes(id))
    }
}
