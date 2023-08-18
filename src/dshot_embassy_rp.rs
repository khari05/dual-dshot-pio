use dshot_encoder as dshot;
pub use super::DualDshotTrait;

use embassy_rp::{
    pio::{ Instance, Pio, Config, PioPin, ShiftConfig, ShiftDirection::Left, InterruptHandler},
    relocate::RelocatedProgram,
    Peripheral, interrupt::typelevel::Binding
};
#[allow(dead_code)]
pub struct DualDshotPio<'a,PIO : Instance> {
    pio_instance: Pio<'a,PIO>
}

#[allow(dead_code)]
impl <'a,PIO: Instance> DualDshotPio<'a,PIO> {
    pub fn new(
        pio: impl Peripheral<P = PIO> + 'a,
        pin0: impl PioPin, pin1: impl PioPin,
        clk_div: (u16, u8),
        _irq: impl Binding<PIO::Interrupt, InterruptHandler<PIO>>
    ) -> DualDshotPio<'a,PIO> {

        // Define program

        let dshot_pio_program = pio_proc::pio_asm!(
            "set pindirs, 1",
            "entry:"
            "   pull"
            "   out null 16"
            "   set x 15"
            "loop:"
            "   set pins 1"
            "   out y 1"
            "   jmp !y zero"
            "   nop [2]"
            "one:" // 6 and 2
            "   set pins 0"
            "   jmp x-- loop"
            "   jmp reset"
            "zero:" // 3 and 5
            "   set pins 0 [3]"
            "   jmp x-- loop"
            "   jmp reset"
            "reset:" // Blank frame
            "   nop [31]"
            "   nop [31]"
            "   nop [31]"
            "   jmp entry [31]"
        );

        let relocated = RelocatedProgram::new(&dshot_pio_program.program);

        // Configure state machines

        let mut cfg = Config::default();
        let mut pio = Pio::new(pio, _irq);
        cfg.use_program(&pio.common.load_program(&relocated), &[]);
        cfg.clock_divider = clk_div.0.into();

        cfg.shift_in = ShiftConfig {
            auto_fill: true,
            direction: Default::default(),
            threshold: 32,
        };

        cfg.shift_out = ShiftConfig {
            auto_fill: Default::default(),
            direction: Left,
            threshold: Default::default(),
        };

        // Set pins and enable all state machines

        let pin0 = pio.common.make_pio_pin(pin0);
        cfg.set_set_pins(&[&pin0]);
        pio.sm0.set_config(&cfg);
        pio.sm0.set_enable(true);

        let pin1 = pio.common.make_pio_pin(pin1);
        cfg.set_set_pins(&[&pin1]);
        pio.sm1.set_config(&cfg);
        pio.sm1.set_enable(true);

        // Return struct of four configured DSHOT state machines
        DualDshotPio { pio_instance : pio }
    }
}

impl <'d,PIO : Instance> super::DualDshotTrait for DualDshotPio<'d,PIO> {

    /// Set the direction of rotation for each motor
    fn reverse(&mut self, reverse: (bool, bool)) -> (bool, bool) {
        self.pio_instance.sm0.tx().push(dshot::reverse(reverse.0) as u32);
        self.pio_instance.sm1.tx().push(dshot::reverse(reverse.1) as u32);
        (true,true) // Embassy PIO does not report failure to push to TX buffer
    }

    /// Set the throttle for each motor. All values are clamped between 48 and 2047
    fn throttle_clamp(&mut self, throttle: (u16, u16)) -> (bool, bool) {
        self.pio_instance.sm0.tx().push(dshot::throttle_clamp(throttle.0, false) as u32);
        self.pio_instance.sm1.tx().push(dshot::throttle_clamp(throttle.1, false) as u32);
        (true,true) // Embassy PIO does not report failure to push to TX buffer
    }

    /// Set the throttle for each motor to zero (Dshot command 48)
    fn throttle_minimum(&mut self) -> (bool, bool) {
        self.pio_instance.sm0.tx().push(dshot::throttle_minimum(false) as u32);
        self.pio_instance.sm1.tx().push(dshot::throttle_minimum(false) as u32);
        (true,true) // Embassy PIO does not report failure to push to TX buffer
    }
}
