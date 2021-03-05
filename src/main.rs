#![no_main]
#![no_std]

use panic_rtt_target as _;

#[rtic::app(device = stm32l4xx_hal::pac, dispatchers = [USART1, USART2])]
mod app {
    use dwt_systick_monotonic::{
        consts::{U0, U80},
        DwtSystick,
    };
    use rtic::time::{duration::*, Instant};
    use rtt_target::{rprintln, rtt_init_print};
    use stm32l4xx_hal::pac::{TIM15, TIM2};
    use stm32l4xx_hal::timer::{ExtendedTimer, Timer};
    use stm32l4xx_hal::{prelude::*, time::Hertz, timer};

    #[monotonic(binds = TIM2)]
    type HalMono = Timer<TIM2>;

    #[monotonic(binds = SysTick, default = true)]
    type DwtMono = DwtSystick<U80, U0, U0>;

    #[monotonic(priority = 4, binds = TIM1_BRK_TIM15)]
    type HalMono2 = ExtendedTimer<TIM15>;

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
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
        let clocks = rcc.cfgr.sysclk(80.mhz()).freeze(&mut flash.acr, &mut pwr);

        // Setup the monotonic timer
        let mono = timer::Timer::free_running_tim2(
            cx.device.TIM2,
            clocks,
            Hertz(80_000_000),
            true,
            &mut rcc.apb1r1,
        );

        // Setup the monotonic timer
        let mono2 = DwtSystick::new(&mut dcb, dwt, systick, clocks.sysclk().0);

        let mono3 = ExtendedTimer::new(timer::Timer::free_running_tim15(
            cx.device.TIM15,
            clocks,
            Hertz(1_000_000),
            true,
            &mut rcc.apb2,
        ));

        rprintln!("init");

        bar::spawn_after(Seconds(1_u32)).ok();
        periodic::spawn(Instant::new(0)).ok();

        (init::LateResources {}, init::Monotonics(mono, mono2, mono3))
    }

    #[task]
    fn foo(_: foo::Context) {
        let now = *DwtMono::now().duration_since_epoch().integer();
        rprintln!("foo (DWT/SysTick): {:?}", now);
        bar::spawn_after(Seconds(1_u32)).ok();
    }

    #[task]
    fn bar(_: bar::Context) {
        // rprintln!("bar (DWT/SysTick)");
        foo::DwtMono::spawn_after(Seconds(1_u32)).ok();
        baz::HalMono::spawn_after(Seconds(1_u32)).ok();
        quox::HalMono2::spawn_after(Seconds(1_u32)).ok();
    }

    #[task]
    fn baz(_: baz::Context) {
        let now = *HalMono::now().duration_since_epoch().integer();
        rprintln!("baz (TIM2): {:?}", now);
    }

    #[task]
    fn quox(_: quox::Context) {
        let now = *HalMono2::now().duration_since_epoch().integer();
        rprintln!("quox (TIM15): {:?}", now);
    }

    #[task(priority = 4)]
    fn periodic(_: periodic::Context, instant: Instant<ExtendedTimer<TIM15>>) {
        let now = *HalMono2::now().duration_since_epoch().integer();
        rprintln!(
            "periodic (TIM15): {:?} / {:?}",
            instant.duration_since_epoch().integer(),
            now
        );

        let next = instant + Milliseconds(900_u32);

        periodic::HalMono2::spawn_at(next, next).ok();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("idle");

        loop {
            cortex_m::asm::nop();
        }
    }
}
