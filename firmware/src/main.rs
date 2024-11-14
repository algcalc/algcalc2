#![no_std]
#![no_main]

mod hardware;

use bsp::entry;
use defmt::info;
use defmt_rtt as _;
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::spi::ExclusiveDevice;
use hardware::Keypad;
use panic_probe as _;

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

use os::hardware::Hardware;

#[entry]
fn main() -> ! {
	info!("Program start");
	let mut pac = pac::Peripherals::take().unwrap();
	let mut watchdog = Watchdog::new(pac.WATCHDOG);
	let sio = Sio::new(pac.SIO);

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
	.unwrap();

	let pins = bsp::Pins::new(
		pac.IO_BANK0,
		pac.PADS_BANK0,
		sio.gpio_bank0,
		&mut pac.RESETS,
	);

	pins.led.into_push_pull_output().set_high().unwrap();

	let eink_mosi = pins.gpio11.into_function::<hal::gpio::FunctionSpi>();
	let eink_sclk = pins.gpio10.into_function::<hal::gpio::FunctionSpi>();
	let eink_spibus = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI1, (eink_mosi, eink_sclk));

	let eink_spibus = eink_spibus.init(
		&mut pac.RESETS,
		clocks.peripheral_clock.freq(),
		16.MHz(),
		embedded_hal::spi::MODE_0,
	);

	let eink_busy = pins
		.gpio13
		.into_function::<hal::gpio::FunctionSpi>()
		.into_floating_input();
	let eink_dc = pins
		.gpio8
		.into_function::<hal::gpio::FunctionSpi>()
		.into_push_pull_output();
	let eink_rst = pins
		.gpio12
		.into_function::<hal::gpio::FunctionSpi>()
		.into_push_pull_output();
	let eink_cs = pins
		.gpio9
		.into_function::<hal::gpio::FunctionSpi>()
		.into_push_pull_output();

	let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
	let eink_spidev = ExclusiveDevice::new(eink_spibus, eink_cs, timer).unwrap();

	let display = hardware::Display::new(eink_spidev, eink_busy, eink_dc, eink_rst, timer);

	let keypad = Keypad::new(
		[
			pins.gpio22.into_pull_down_input().into_dyn_pin(),
			pins.gpio21.into_pull_down_input().into_dyn_pin(),
			pins.gpio20.into_pull_down_input().into_dyn_pin(),
			pins.gpio19.into_pull_down_input().into_dyn_pin(),
			pins.gpio18.into_pull_down_input().into_dyn_pin(),
		],
		[
			pins.gpio14.into_push_pull_output().into_dyn_pin(),
			pins.gpio15.into_push_pull_output().into_dyn_pin(),
			pins.gpio16.into_push_pull_output().into_dyn_pin(),
			pins.gpio17.into_push_pull_output().into_dyn_pin(),
		],
		timer,
	);

	let hw = Hardware {
		display,
		keypad,
		system: hardware::System,
	};

	os::run(hw)
}
