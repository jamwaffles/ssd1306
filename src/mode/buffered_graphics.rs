//! Buffered graphics mode.

use crate::{
    command::AddrMode,
    mode::DisplayConfig,
    rotation::DisplayRotation,
    size::{DisplaySize, NewZeroed},
    Ssd1306,
};
use display_interface::{DisplayError, WriteOnlyDataCommand};

/// Buffered graphics mode.
///
/// This mode keeps a pixel buffer in system memory, up to 1024 bytes for 128x64px displays. This
/// buffer is drawn to by [`set_pixel`](Ssd1306::set_pixel) commands or
/// [`embedded-graphics`](https://docs.rs/embedded-graphics) commands. The display can then be
/// updated using the [`flush`](Ssd1306::flush) method.
#[derive(Clone, Debug)]
pub struct BufferedGraphicsMode<SIZE>
where
    SIZE: DisplaySize,
{
    buffer: SIZE::Buffer,
    min_x: u8,
    max_x: u8,
    min_y: u8,
    max_y: u8,
}

impl<SIZE> BufferedGraphicsMode<SIZE>
where
    SIZE: DisplaySize,
{
    /// Create a new buffered graphics mode instance.
    pub(crate) fn new() -> Self {
        Self {
            buffer: NewZeroed::new_zeroed(),
            min_x: 255,
            max_x: 0,
            min_y: 255,
            max_y: 0,
        }
    }
}

impl<DI, SIZE> DisplayConfig for Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    type Error = DisplayError;

    /// Set the display rotation
    ///
    /// This method resets the cursor but does not clear the screen.
    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_rotation(rot)
    }

    /// Initialise and clear the display in graphics mode.
    fn init(&mut self) -> Result<(), DisplayError> {
        self.clear_impl(false);
        self.init_with_addr_mode(AddrMode::Horizontal)
    }
}

impl<DI, SIZE> Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    SIZE: DisplaySize,
{
    fn clear_impl(&mut self, value: bool) {
        self.mode.buffer.as_mut().fill(value as u8);

        let (width, height) = self.dimensions();
        self.mode.min_x = 0;
        self.mode.max_x = width - 1;
        self.mode.min_y = 0;
        self.mode.max_y = height - 1;
    }

    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: bool) {
        let value = value as u8;
        let rotation = self.rotation;

        let (idx, bit) = match rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                let idx = ((y as usize) / 8 * SIZE::WIDTH as usize) + (x as usize);
                let bit = y % 8;

                (idx, bit)
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                let idx = ((x as usize) / 8 * SIZE::WIDTH as usize) + (y as usize);
                let bit = x % 8;

                (idx, bit)
            }
        };

        if let Some(byte) = self.mode.buffer.as_mut().get_mut(idx) {
            // Keep track of max and min values
            self.mode.min_x = self.mode.min_x.min(x as u8);
            self.mode.max_x = self.mode.max_x.max(x as u8);

            self.mode.min_y = self.mode.min_y.min(y as u8);
            self.mode.max_y = self.mode.max_y.max(y as u8);

            // Set pixel value in byte
            // Ref this comment https://stackoverflow.com/questions/47981/how-do-you-set-clear-and-toggle-a-single-bit#comment46654671_47990
            *byte = *byte & !(1 << bit) | (value << bit)
        }
    }

    fn dirty_area(&self, width: u8, height: u8) -> ((u8, u8), (u8, u8)) {
        let min = (self.mode.min_x, self.mode.min_y);
        let max = match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (
                (self.mode.max_x + 1).min(width),
                (self.mode.max_y | 7).min(height),
            ),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (
                (self.mode.max_x | 7).min(width),
                (self.mode.max_y + 1).min(height),
            ),
        };

        (min, max)
    }

    fn display_area(&self, disp_min: (u8, u8), disp_max: (u8, u8)) -> ((u8, u8), (u8, u8)) {
        let offset_x = match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate270 => SIZE::OFFSETX,
            DisplayRotation::Rotate180 | DisplayRotation::Rotate90 => {
                // If segment remapping is flipped, we need to calculate
                // the offset from the other edge of the display.
                SIZE::DRIVER_COLS - SIZE::WIDTH - SIZE::OFFSETX
            }
        };

        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (
                (disp_min.0 + offset_x, disp_min.1 + SIZE::OFFSETY),
                (disp_max.0 + offset_x, disp_max.1 + SIZE::OFFSETY),
            ),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (
                (disp_min.1 + offset_x, disp_min.0 + SIZE::OFFSETY),
                (disp_max.1 + offset_x, disp_max.0 + SIZE::OFFSETY),
            ),
        }
    }
}

impl<DI, SIZE> Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    /// Write out data to a display.
    ///
    /// This only updates the parts of the display that have changed since the last flush.
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        // Nothing to do if no pixels have changed since the last update
        if self.mode.max_x < self.mode.min_x || self.mode.max_y < self.mode.min_y {
            return Ok(());
        }

        let (width, height) = self.dimensions();
        let (disp_min, disp_max) = self.dirty_area(width, height);
        let (area_start, area_end) = self.display_area(disp_min, disp_max);

        self.mode.min_x = 255;
        self.mode.max_x = 0;
        self.mode.min_y = 255;
        self.mode.max_y = 0;

        // Tell the display to update only the part that has changed
        self.set_draw_area(area_start, area_end)?;

        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => Self::flush_buffer_chunks(
                &mut self.interface,
                self.mode.buffer.as_mut(),
                width as usize,
                (disp_min.0, disp_min.1),
                (disp_max.0, disp_max.1),
            ),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => Self::flush_buffer_chunks(
                &mut self.interface,
                self.mode.buffer.as_mut(),
                height as usize,
                (disp_min.1, disp_min.0),
                (disp_max.1, disp_max.0),
            ),
        }
    }
}

#[cfg(feature = "async")]
impl<DI, SIZE> Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    DI: display_interface::AsyncWriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    /// Write out data to a display.
    ///
    /// This only updates the parts of the display that have changed since the last flush.
    pub async fn flush_async(&mut self) -> Result<(), DisplayError> {
        // Nothing to do if no pixels have changed since the last update
        if self.mode.max_x < self.mode.min_x || self.mode.max_y < self.mode.min_y {
            return Ok(());
        }

        let (width, height) = self.dimensions();
        let (disp_min, disp_max) = self.dirty_area(width, height);
        let (area_start, area_end) = self.display_area(disp_min, disp_max);

        self.mode.min_x = 255;
        self.mode.max_x = 0;
        self.mode.min_y = 255;
        self.mode.max_y = 0;

        // Tell the display to update only the part that has changed
        self.set_draw_area_async(area_start, area_end).await?;

        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                Self::flush_buffer_chunks_async(
                    &mut self.interface,
                    self.mode.buffer.as_mut(),
                    width as usize,
                    (disp_min.0, disp_min.1),
                    (disp_max.0, disp_max.1),
                )
                .await
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                Self::flush_buffer_chunks_async(
                    &mut self.interface,
                    self.mode.buffer.as_mut(),
                    height as usize,
                    (disp_min.1, disp_min.0),
                    (disp_max.1, disp_max.0),
                )
                .await
            }
        }
    }
}

#[cfg(feature = "graphics")]
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    geometry::{Dimensions, OriginDimensions},
    pixelcolor::BinaryColor,
    Pixel,
};

#[cfg(feature = "graphics")]
impl<DI, SIZE> DrawTarget for Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    SIZE: DisplaySize,
{
    type Color = BinaryColor;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| {
                self.set_pixel(pos.x as u32, pos.y as u32, color.is_on())
            });

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.clear_impl(color.is_on());
        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<DI, SIZE> OriginDimensions for Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    SIZE: DisplaySize,
{
    fn size(&self) -> Size {
        let (w, h) = self.dimensions();

        Size::new(w.into(), h.into())
    }
}
