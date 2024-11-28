#![no_std]
#![no_main]

mod config;
mod hardware;
mod input_thread;

use bsp::entry;
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use hardware::Keypad;
use input_thread::KeypadPins;
use panic_probe as _;

use rp_pico::{
	self as bsp,
	hal::{
		self,
		adc::AdcPin,
		multicore::{Multicore, Stack},
		Adc, Timer,
	},
};

use bsp::hal::{
	clocks::{init_clocks_and_plls, Clock},
	fugit::RateExtU32,
	pac,
	sio::Sio,
	watchdog::Watchdog,
};

use os::hardware::Hardware;

static mut CORE1_STACK: Stack<4096> = Stack::new();

#[entry]
fn main() -> ! {
	let mut pac = pac::Peripherals::take().unwrap();
	let mut watchdog = Watchdog::new(pac.WATCHDOG);
	let mut sio = Sio::new(pac.SIO);

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

	// POWER LED

	#[cfg(feature = "powerled")]
	pins.led.into_push_pull_output().set_high().unwrap();

	// EINK

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

	// KEYPAD

	let keypad_pins = KeypadPins {
		columns: [
			pins.gpio22.into_pull_down_input().into_dyn_pin(),
			pins.gpio21.into_pull_down_input().into_dyn_pin(),
			pins.gpio20.into_pull_down_input().into_dyn_pin(),
			pins.gpio19.into_pull_down_input().into_dyn_pin(),
			pins.gpio18.into_pull_down_input().into_dyn_pin(),
		],
		rows: [
			pins.gpio14.into_push_pull_output().into_dyn_pin(),
			pins.gpio15.into_push_pull_output().into_dyn_pin(),
			pins.gpio16.into_push_pull_output().into_dyn_pin(),
			pins.gpio17.into_push_pull_output().into_dyn_pin(),
		],
	};

	let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
	let cores = mc.cores();
	let secondary_core = &mut cores[1];
	#[allow(static_mut_refs)] // multicore is unsound. literally.
	let _ = secondary_core.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
		input_thread::run(keypad_pins, timer);
	});

	let keypad = Keypad::new(sio.fifo, timer);

	// BATTERY

	let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
	let mut adc_pin_battery = AdcPin::new(pins.gpio28.into_floating_input()).unwrap();
	let mut _adc_fifo = adc
		.build_fifo()
		.clock_divider(0xFFFF, 0)
		.set_channel(&mut adc_pin_battery)
		.start_paused();
	// loop {
	// 	adc_fifo.resume();
	// 	let mut counts_sum = 0u32;
	// 	for _ in 0..1648000 {
	// 		counts_sum += adc_fifo.read() as u32;
	// 	}
	// 	adc_fifo.pause();
	// 	adc_fifo.clear();
	// 	let avg_counts = counts_sum / 1648000;
	// 	defmt::debug!(
	// 		"avgcounts: {}, percentage: {}%",
	// 		avg_counts,
	// 		(avg_counts - 1862) * 100 / 744
	// 	);
	// 	timer.delay_ms(10000);
	// }

	let hw = Hardware {
		display,
		keypad,
		system: hardware::System,
	};

	os::run(hw)
}
