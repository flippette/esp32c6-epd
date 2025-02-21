#![no_std]
#![no_main]
#![feature(generic_arg_infer, impl_trait_in_assoc_type)]
#![expect(unstable_features)]

mod error;
mod macros;
mod rt;

use defmt::{info, intern};
use defmt_rtt as _;
use display_interface_spi::SPIInterface;
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use embedded_graphics::prelude::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use error::{Report, ResultExt};
use esp_backtrace as _;
use esp_hal::{
    Async,
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Input, Level, Output, Pull},
    spi::master::{self, Spi, SpiDmaBus},
    time::RateExtU32,
    timer::systimer::SystemTimer,
};
use tinybmp::Bmp;
use weact_studio_epd::{
    Color, DisplayDriver,
    graphics::{self, Display},
};

type Bw420Driver = DisplayDriver<
    SPIInterface<
        ExclusiveDevice<SpiDmaBus<'static, Async>, Output<'static>, Delay>,
        Output<'static>,
    >,
    Input<'static>,
    Output<'static>,
    Delay,
    400,
    400,
    300,
    Color,
>;
type Bw420Frame =
    Display<400, 300, { graphics::buffer_len::<Color>(400, 300) }, Color>;

static FRAMES: &[&[u8]] = frames::embed!("assets/video.mp4");

async fn main(_s: Spawner) -> Result<(), Report<8>> {
    let p = esp_hal::init(<_>::default());
    esp_hal_embassy::init(SystemTimer::new(p.SYSTIMER).alarm0);
    info!("HAL init!");

    let mut epd = {
        #[allow(clippy::manual_div_ceil)]
        let (tx_buf, tx_desc, rx_buf, rx_desc) = dma_buffers!(4096);
        let spi = Spi::new(
            p.SPI2,
            master::Config::default().with_frequency(40.MHz()),
        )
        .into_report()?
        .with_mosi(p.GPIO4)
        .with_sck(p.GPIO5)
        .with_dma(p.DMA_CH0)
        .with_buffers(
            DmaRxBuf::new(rx_desc, rx_buf).into_report()?,
            DmaTxBuf::new(tx_desc, tx_buf).into_report()?,
        )
        .into_async();

        let cs = Output::new(p.GPIO6, Level::Low);
        let Ok(dev) = ExclusiveDevice::new(spi, cs, Delay);
        let dc = Output::new(p.GPIO7, Level::Low);
        let busy = Input::new(p.GPIO1, Pull::Down);
        let reset = Output::new(p.GPIO0, Level::Low);

        Bw420Driver::new(SPIInterface::new(dev, dc), busy, reset, Delay)
    };

    epd.init().await.into_report()?;
    epd.clear_bw_buffer().await.into_report()?;
    epd.full_refresh().await.into_report()?;
    info!("display cleared!");

    let mut fb = Bw420Frame::new();
    fb.clear(Color::White);

    for (nth, frame) in FRAMES.iter().rev().enumerate() {
        let img = Bmp::<Color>::from_slice(frame)
            .map_err(|_| intern!("BMP conversion error!"))
            .into_report()?;
        fb.draw_iter(img.pixels().map(|px| {
            Pixel(
                px.0,
                match px.1 {
                    Color::White => Color::Black,
                    Color::Black => Color::White,
                },
            )
        }))
        .unwrap();

        epd.full_update(&fb).await.into_report()?;
        info!("drew frame #{=usize}", nth);

        Timer::after_secs(5).await;
    }

    Ok(())
}
