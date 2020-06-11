//! Display size

use super::command::Command;
use typenum::{Unsigned, U0, U16, U28, U32, U40, U48, U64, U72, U96, U128};

pub trait DisplaySize {
    type Width: Unsigned;
    type Height: Unsigned;
    type OffsetX: Unsigned;
    type OffsetY: Unsigned;

    fn ComPinConfig() -> Command;
}

pub struct DisplaySize128x64;
impl DisplaySize for DisplaySize128x64 {
    type Width = U128;
    type Height = U64;
    type OffsetX = U0;
    type OffsetY = U0;

    fn ComPinConfig() -> Command {
        Command::ComPinConfig(true, false)
    }
}

pub struct DisplaySize128x32;
impl DisplaySize for DisplaySize128x32 {
    type Width = U128;
    type Height = U32;
    type OffsetX = U0;
    type OffsetY = U0;

    fn ComPinConfig() -> Command {
        Command::ComPinConfig(false, false)
    }
}

pub struct DisplaySize96x16;
impl DisplaySize for DisplaySize96x16 {
    type Width = U96;
    type Height = U16;
    type OffsetX = U0;
    type OffsetY = U0;

    fn ComPinConfig() -> Command {
        Command::ComPinConfig(false, false)
    }
}

pub struct DisplaySize72x40;
impl DisplaySize for DisplaySize72x40 {
    type Width = U72;
    type Height = U40;
    type OffsetX = U28;
    type OffsetY = U0;

    fn ComPinConfig() -> Command {
        Command::ComPinConfig(true, false)
    }
}

pub struct DisplaySize64x48;
impl DisplaySize for DisplaySize64x48 {
    type Width = U64;
    type Height = U48;
    type OffsetX = U32;
    type OffsetY = U0;

    fn ComPinConfig() -> Command {
        Command::ComPinConfig(true, false)
    }
}
