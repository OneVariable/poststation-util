use core::sync::atomic::{compiler_fence, Ordering};

use postcard_rpc::header::VarHeader;

use crate::app::Context;

/// This is an example of a BLOCKING handler.
pub fn unique_id(context: &mut Context, _header: VarHeader, _arg: ()) -> u64 {
    context.unique_id
}

pub fn picoboot_reset(_context: &mut Context, _header: VarHeader, _arg: ()) {
    embassy_rp::rom_data::reset_to_usb_boot(0, 0);
    loop {
        // Wait for reset...
        compiler_fence(Ordering::SeqCst);
    }
}
