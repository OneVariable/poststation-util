use core::sync::atomic::{compiler_fence, Ordering};

use embassy_time::{Instant, Timer};
use postcard_rpc::{header::VarHeader, server::Sender};
use i2c_passthru_icd::{I2cError, LedState, ReadCommand, ReadData, ReadResult, SleepEndpoint, SleepMillis, SleptMillis, WriteCommand, WriteResult};

use crate::app::{AppTx, Context, TaskContext};

/// This is an example of a BLOCKING handler.
pub fn unique_id(context: &mut Context, _header: VarHeader, _arg: ()) -> u64 {
    context.unique_id
}

/// Also a BLOCKING handler
pub fn picoboot_reset(_context: &mut Context, _header: VarHeader, _arg: ()) {
    embassy_rp::rom_data::reset_to_usb_boot(0, 0);
    loop {
        // Wait for reset...
        compiler_fence(Ordering::SeqCst);
    }
}

/// Also a BLOCKING handler
pub fn set_led(context: &mut Context, _header: VarHeader, arg: LedState) {
    match arg {
        LedState::Off => context.led.set_low(),
        LedState::On => context.led.set_high(),
    }
}

pub fn get_led(context: &mut Context, _header: VarHeader, _arg: ()) -> LedState {
    match context.led.is_set_low() {
        true => LedState::Off,
        false => LedState::On,
    }
}

pub async fn i2c_read(context: &mut Context, _header: VarHeader, arg: ReadCommand) -> ReadResult<'_> {
    let len = arg.len as usize;
    if len > context.buf.len() {
        return Err(I2cError)
    }
    let Context { i2c, buf, .. } = context;
    let buf = &mut buf[..len];
    let res = i2c.read_async(arg.addr, buf).await;
    match res {
        Ok(()) => Ok(ReadData { data: buf }),
        Err(_e) => Err(I2cError),
    }
}

pub async fn i2c_write(context: &mut Context, _header: VarHeader, arg: WriteCommand<'_>) -> WriteResult {
    context.i2c.write_async(arg.addr, arg.data.iter().copied())
        .await
        .map_err(|_| I2cError)
}

/// This is a SPAWN handler
///
/// The pool size of three means we can have up to three of these requests "in flight"
/// at the same time. We will return an error if a fourth is requested at the same time
#[embassy_executor::task(pool_size = 3)]
pub async fn sleep_handler(_context: TaskContext, header: VarHeader, arg: SleepMillis, sender: Sender<AppTx>) {
    // We can send string logs, using the sender
    let _ = sender.log_str("Starting sleep...").await;
    let start = Instant::now();
    Timer::after_millis(arg.millis.into()).await;
    let _ = sender.log_str("Finished sleep").await;
    // Async handlers have to manually reply, as embassy doesn't support returning by value
    let _ = sender.reply::<SleepEndpoint>(header.seq_no, &SleptMillis { millis: start.elapsed().as_millis() as u16 }).await;
}
