#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m::asm;
use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use embedded_hal::digital::StatefulOutputPin;
use embedded_hal::prelude::*;
use flash_embedded_hal::flash::{Locking, WriteErase};
use nb::block;
use nucleo_l011k4_bsp as bsp;
use stm32l0x1_hal as hal;
use stm32l0x1_hal::time::Hertz;

#[entry]
fn main() -> ! {
    let _p = cortex_m::Peripherals::take().unwrap();
    let d = hal::stm32l0x1::Peripherals::take().unwrap();

    let mut board = bsp::init::<hal::power::VCoreRange1>(d.PWR, d.FLASH, d.RCC);

    //let ticks = board.rcc.cfgr.context().unwrap().sysclk().0;
    //board.systick_start(&mut p.SYST, SystClkSource::Core, ticks / 1000);

    let pins = board.pins(d.GPIOA, d.GPIOB, d.GPIOC);

    let mut user_led = board.user_led(pins.d13);

    let mut timer = hal::timer::Timer::tim2(
        d.TIM2,
        board.rcc.cfgr.context().unwrap(),
        Hertz(1),
        &mut board.rcc.apb1,
    );

    let (mut vcp_tx, mut vcp_rx) = board
        .vcp_usart(
            d.USART2,
            pins.a7,
            hal::rcc::clocking::USARTClkSource::SYSCLK,
        )
        .split();

    let mut i = 0;
    loop {
        timer.try_start(Hertz(1)).unwrap();
        block!(timer.try_wait()).unwrap();
        if user_led.try_is_set_high().unwrap() {
            user_led.try_set_low().unwrap();
        } else {
            user_led.try_set_high().unwrap();
        }

        board.flash.unlock();
        board.flash.erase_page(0x3000 as usize).unwrap();
        unsafe {
            if *(0x3000 as *const u32) != 0 {
                asm::bkpt();
            }
        }
        board.flash.program_word(0x3000 as usize, i).unwrap();
        unsafe {
            if *(0x3000 as *const u32) != i {
                asm::bkpt();
            }
        }
        board.flash.lock();

        vcp_tx.try_write(vcp_rx.try_read().unwrap()).unwrap();

        i += 1;
    }
}

#[exception]
fn SysTick() {
    //asm::bkpt();
}

#[exception]
fn HardFault(_ef: &ExceptionFrame) -> ! {
    //panic!("HardFault at {:#?}", ef);
    panic!("Hardfault");
}

#[exception]
fn DefaultHandler(_irqn: i16) {
    //panic!("Unhandled exception (IRQn = {})", irqn);
    panic!("Unhandled exception");
}
