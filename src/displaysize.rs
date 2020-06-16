//! Display size

// No need to implement Copy
#![allow(missing_copy_implementations)]

use super::command::Command;
use display_interface::{DisplayError, WriteOnlyDataCommand};
use generic_array::ArrayLength;
use typenum::{U1024, U192, U360, U384, U512};

/// Display information
///
/// This trait describes information related to a particular display.
/// This includes resolution, offset and framebuffer size.
pub trait DisplaySize {
    /// Width in pixels
    const WIDTH: u8;

    /// Height in pixels
    const HEIGHT: u8;

    /// Horizontal offset in pixels
    const OFFSETX: u8 = 0;

    /// Vertical offset in pixels
    const OFFSETY: u8 = 0;

    /// Size of framebuffer. Because the display is monocrome, this is
    /// width * height / 8
    type BufferSize: ArrayLength<u8>;

    /// Send resolution-dependent configuration to the display
    ///
    /// See [`Command::ComPinConfig`](../command/enum.Command.html#variant.ComPinConfig)
    /// for more information
    fn configure(&self, iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError>;
}

/// Size information for the common 128x64 variants
pub struct DisplaySize128x64;
impl DisplaySize for DisplaySize128x64 {
    const WIDTH: u8 = 128;
    const HEIGHT: u8 = 64;
    type BufferSize = U1024;

    fn configure(&self, iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError> {
        Command::ComPinConfig(true, false).send(iface)
    }
}

/// Size information for the common 128x32 variants
pub struct DisplaySize128x32;
impl DisplaySize for DisplaySize128x32 {
    const WIDTH: u8 = 128;
    const HEIGHT: u8 = 32;
    type BufferSize = U512;

    fn configure(&self, iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError> {
        Command::ComPinConfig(false, false).send(iface)
    }
}

/// Size information for the common 96x16 variants
pub struct DisplaySize96x16;
impl DisplaySize for DisplaySize96x16 {
    const WIDTH: u8 = 96;
    const HEIGHT: u8 = 16;
    type BufferSize = U192;

    fn configure(&self, iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError> {
        Command::ComPinConfig(false, false).send(iface)
    }
}

/// Size information for the common 72x40 variants
pub struct DisplaySize72x40;
impl DisplaySize for DisplaySize72x40 {
    const WIDTH: u8 = 72;
    const HEIGHT: u8 = 40;
    const OFFSETX: u8 = 28;
    const OFFSETY: u8 = 0;
    type BufferSize = U360;

    fn configure(&self, iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError> {
        Command::ComPinConfig(true, false).send(iface)
    }
}

/// Size information for the common 64x48 variants
pub struct DisplaySize64x48;
impl DisplaySize for DisplaySize64x48 {
    const WIDTH: u8 = 64;
    const HEIGHT: u8 = 48;
    const OFFSETX: u8 = 32;
    const OFFSETY: u8 = 0;
    type BufferSize = U384;

    fn configure(&self, iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError> {
        Command::ComPinConfig(true, false).send(iface)
    }
}
