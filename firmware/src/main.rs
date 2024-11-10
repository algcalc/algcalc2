#![no_std]
#![no_main]

mod hardware;

use bsp::entry;
use defmt::info;
use embedded_hal_bus::spi::ExclusiveDevice;

use rp_pico::{
	self as bsp,
	hal::{self, Timer},
};

use bsp::hal::{
	clocks::{init_clocks_and_plls, Clock},
	fugit::RateExtU32,
	pac,
	sio::Sio,
	watchdog::Watchdog,
};

use crate::hardware::Display;
use embedded_graphics::prelude::*;
use epd_waveshare::epd2in9_v2::{Display2in9, Epd2in9};
use epd_waveshare::prelude::WaveshareDisplay;
use os::hardware::{DisplayDriver, Hardware};

#[entry]
fn main() -> ! {
	info!("Program start");
	let mut pac = pac::Peripherals::take().unwrap();
	let _core = pac::CorePeripherals::take().unwrap();
	let mut watchdog = Watchdog::new(pac.WATCHDOG);
	let sio = Sio::new(pac.SIO);

	// External high-speed crystal on the pico board is 12Mhz
	let external_xtal_freq_hz = 12_000_000u32;
	let clocks = init_clocks_and_plls(
		external_xtal_freq_hz,
		pac.XOSC,
		pac.CLOCKS,
		pac.PLL_SYS,
		pac.PLL_USB,
		&mut pac.RESETS,
		&mut watchdog,
	)
	.ok()
	.unwrap();

	let pins = bsp::Pins::new(
		pac.IO_BANK0,
		pac.PADS_BANK0,
		sio.gpio_bank0,
		&mut pac.RESETS,
	);

	let _led_pin = pins.led.into_push_pull_output();

	let spi_mosi = pins.gpio11.into_function::<hal::gpio::FunctionSpi>();
	/* let spi_miso = pins.gpio12.into_function::<hal::gpio::FunctionSpi>(); */
	let spi_sclk = pins.gpio10.into_function::<hal::gpio::FunctionSpi>();
	let spi = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI1, (spi_mosi, spi_sclk));

	// Exchange the uninitialised SPI driver for an initialised one
	let spi = spi.init(
		&mut pac.RESETS,
		clocks.peripheral_clock.freq(),
		8.MHz(),
		embedded_hal::spi::MODE_0,
	);

	let busy_in = pins
		.gpio13
		.into_function::<hal::gpio::FunctionSpi>()
		.into_floating_input();
	let dc = pins
		.gpio8
		.into_function::<hal::gpio::FunctionSpi>()
		.into_push_pull_output();
	let rst = pins
		.gpio12
		.into_function::<hal::gpio::FunctionSpi>()
		.into_push_pull_output();
	let cs = pins
		.gpio9
		.into_function::<hal::gpio::FunctionSpi>()
		.into_push_pull_output();

	let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
	let mut spi = ExclusiveDevice::new(spi, cs, timer.clone()).unwrap();

	let epd = Epd2in9::new(&mut spi, busy_in, dc, rst, &mut timer, None).unwrap();
	let display = Display {
		epd,
		fb: Display2in9::default()
	};

	let hw = Hardware {
		display,
		keypad: todo!()
	};

	os::run(hw);

	loop {}
}
