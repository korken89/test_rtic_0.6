#![no_main]
#![no_std]

use panic_rtt_target as _;

#[rtic::app(device = stm32l4xx_hal::pac, dispatchers = [USART1, USART2])]
mod app {
    // use dwt_systick_monotonic::DwtSystick;
    // use rtic::time::duration::*;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32l4xx_hal::prelude::*;

    #[local]
    struct Local {
        a: u32,
    }

    #[shared]
    struct Shared {
        b: i32,
    }

    // #[monotonic(binds = SysTick, default = true)]
    // type DwtMono = DwtSystick<80_000_000>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);
        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        rtt_init_print!(NoBlockSkip, 4096);

        rprintln!("pre init");

        //
        // Initialize the clocks
        //
        let _clocks = rcc.cfgr.sysclk(80.mhz()).freeze(&mut flash.acr, &mut pwr);

        // Setup the monotonic timer
        // let mono2 = DwtSystick::new(&mut dcb, dwt, systick, clocks.sysclk().0);

        rprintln!("init");

        t1::spawn().unwrap();

        (Shared { b: 3 }, Local { a: 1 }, init::Monotonics {})
    }

    #[task(shared = [b])]
    fn t1(mut cx: t1::Context) {
        cx.shared.b.lock(|b| *b += 5);

    }

    #[task(local = [a])]
    fn t2(cx: t2::Context) {
        *cx.local.a += 2;
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("idle");

        loop {
            cortex_m::asm::nop();
        }
    }
}
