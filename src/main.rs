#![no_std]
#![no_main]

use stm32h7xx_hal::gpio::{gpiof, ExtiPin, Output, PushPull};
use stm32h7xx_hal::pac::syscfg;
use stm32h7xx_hal::pwr::VoltageScale;
use stm32h7xx_hal::{interrupt, pac, prelude::*};
use stm32h7xx_hal::gpio::{gpiod, Edge, Input};
use cortex_m_rt::entry;
use panic_halt as _;

use stm32h7xx_hal::gpio::PinState::{High, Low};

use core::cell::{Cell, RefCell};
use cortex_m::interrupt::{free, Mutex};
use cortex_m::peripheral::NVIC;

static SEMAPHORE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

static ENCODER1_PIN: Mutex<RefCell<Option<gpiod::PD0<Input>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<gpiof::PF8<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> !{
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(480.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    let mut syscfg = dp.SYSCFG;
    let mut exti = dp.EXTI;

    let gpiof = dp.GPIOF.split(ccdr.peripheral.GPIOF);
    let mut led = gpiof.pf8.into_push_pull_output();


    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
    let mut encoder1 = gpiod.pd0.into_pull_up_input();
    encoder1.make_interrupt_source(&mut syscfg);
    encoder1.trigger_on_edge(&mut exti, Edge::Rising);
    encoder1.enable_interrupt(&mut exti);

    free(|cs| {
            ENCODER1_PIN.borrow(cs).replace(Some(encoder1));
            LED.borrow(cs).replace(Some(led));
            
    });

    unsafe {
        cp.NVIC.set_priority(interrupt::EXTI3, 1);
        //cp.NVIC.set_priority(interrupt::EXTI9_5, 1);
        NVIC::unmask::<interrupt>(interrupt::EXTI3);
        //NVIC::unmask::<interrupt>(interrupt::EXTI9_5);
    }
    
    loop {
        cortex_m::asm::nop();
    }

fn toggle_led(){
    free(|cs|{
        if let Some(b) = LED.borrow(cs).borrow_mut().as_mut() {
            let led_state = b.get_state();

            if led_state == High {
                b.set_low();
            }else {
                b.set_high();
            }
        } 
    })
}

fn EXTI3() {
    toggle_led();

    free(|cs|{
        if let Some(b) = ENCODER1_PIN.borrow(cs).borrow_mut().as_mut() {
            b.clear_interrupt_pending_bit();
        }

        SEMAPHORE.borrow(cs).set(false);
    });
}

}
