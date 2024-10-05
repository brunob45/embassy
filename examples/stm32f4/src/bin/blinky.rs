#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::pac;

pub enum CaptureInputPolarity {
    RisingEdge,
    FallingEdge,
    Reserved,
    BothEdge,
}

#[derive(Clone, Copy)]
pub struct TimerCaptureCompare {
    regs: *mut(),
    ch: usize,
}

impl TimerCaptureCompare {
    fn regs_core(self) -> pac::timer::TimGp16 {
        unsafe { crate::pac::timer::TimGp16::from_ptr(self.regs) }
    }
    pub fn write_dma_request_enable(&self, val: bool) {
        self.regs_core().dier().modify(|r| r.set_ccde(self.ch, val));
    }
    pub fn read_dma_request_enable(&self) -> bool {
        self.regs_core().dier().read().ccde(self.ch)
    }
    pub fn write_interrupt_enable(&self, val: bool) {
        self.regs_core().dier().modify(|r| r.set_ccie(self.ch, val));
    }
    pub fn read_interrupt_enable(&self) -> bool {
        self.regs_core().dier().read().ccie(self.ch)
    }
    pub fn write_interrupt_flag(&self) {
        // different behavior capture compare !
        self.regs_core().sr().modify(|r| r.set_ccif(self.ch, false)); // cleared by writing 0
    }
    pub fn read_interrupt_flag(&self) -> bool {
        // different behavior capture compare !
        self.regs_core().sr().read().ccif(self.ch)
    }
    pub fn write_overcapture_flag(&self) {
        self.regs_core().sr().modify(|r| r.set_ccof(self.ch, false)); // cleared by writing 0
    }
    pub fn read_overcapture_flag(&self) -> bool {
        self.regs_core().sr().read().ccof(self.ch)
    }
    pub fn write_generation(&self) {
        // different behavior capture compare !
        // egr is write only !
        self.regs_core().egr().write(|r| r.set_ccg(self.ch, true));
    }
    // output mode
    fn ccmr_output(&self) -> pac::common::Reg<pac::timer::regs::CcmrOutputGp16, pac::common::RW> {
        self.regs_core().ccmr_output(self.ch / 2)
    }
    pub fn write_selection_output(&self, val: pac::timer::vals::CcmrOutputCcs) {
        self.ccmr_output().modify(|r| r.set_ccs(self.ch % 2, val))
    }
    pub fn read_selection_output(&self) -> pac::timer::vals::CcmrOutputCcs {
        self.ccmr_output().read().ccs(self.ch % 2)
    }
    pub fn write_fast_enable(&self, val: bool) {
        self.ccmr_output().modify(|r| r.set_ocfe(self.ch % 2, val))
    }
    pub fn read_fast_enable(&self) -> bool {
        self.ccmr_output().read().ocfe(self.ch % 2)
    }
    pub fn write_preload_enable(&self, val: bool) {
        self.ccmr_output().modify(|r| r.set_ocpe(self.ch % 2, val));
    }
    pub fn read_preload_enable(&self) -> bool {
        self.ccmr_output().read().ocpe(self.ch % 2)
    }
    pub fn write_mode(&self, val: pac::timer::vals::Ocm) {
        self.ccmr_output().modify(|r| r.set_ocm(self.ch % 2, val));
    }
    pub fn read_mode(&self) -> pac::timer::vals::Ocm {
        self.ccmr_output().read().ocm(self.ch % 2)
    }
    pub fn write_clear_enable(&self, val: bool) {
        self.ccmr_output().modify(|r| r.set_occe(self.ch % 2, val))
    }
    pub fn read_clear_enable(&self) -> bool {
        self.ccmr_output().read().occe(self.ch % 2)
    }
    // input mode
    fn ccmr_input(&self) -> pac::common::Reg<pac::timer::regs::CcmrInput2ch, pac::common::RW> {
        self.regs_core().ccmr_input(self.ch / 2)
    }
    pub fn write_selection_input(&self, val: pac::timer::vals::CcmrInputCcs) {
        self.ccmr_input().modify(|r| r.set_ccs(self.ch % 2, val))
    }
    pub fn read_selection_input(&self) -> pac::timer::vals::CcmrInputCcs {
        self.ccmr_input().read().ccs(self.ch % 2)
    }
    pub fn write_prescaler(&self, val: u8) {
        // 0=every event, 1=every 2 events, 2=every 4 events, 3=every 8 events
        self.ccmr_input().modify(|r| r.set_icpsc(self.ch % 2, val));
    }
    pub fn read_prescaler(&self) -> u8 {
        self.ccmr_input().read().icpsc(self.ch % 2)
    }
    pub fn write_filter(&self, val: pac::timer::vals::FilterValue) {
        self.ccmr_input().modify(|r| r.set_icf(self.ch % 2, val))
    }
    pub fn read_filter(&self) -> pac::timer::vals::FilterValue {
        self.ccmr_input().read().icf(self.ch % 2)
    }
    pub fn write_output_enable(&self, val: bool) {
        self.regs_core().ccer().modify(|r| r.set_cce(self.ch, val))
    }
    pub fn read_output_enable(&self) -> bool {
        self.regs_core().ccer().read().cce(self.ch)
    }
    pub fn write_capture_enable(&self, val: bool) {
        self.write_output_enable(val);
    }
    pub fn read_capture_enable(&self) -> bool {
        self.read_output_enable()
    }
    pub fn write_output_polarity(&self, val: bool) {
        // only ouput
        self.regs_core().ccer().modify(|r| r.set_ccp(self.ch, val));
    }
    pub fn read_output_polarity(&self) -> bool {
        // only ouput
        self.regs_core().ccer().read().ccp(self.ch)
    }
    pub fn write_input_polarity(&self, val: CaptureInputPolarity) {
        self.regs_core().ccer().modify(|r| match val {
                CaptureInputPolarity::RisingEdge => {
                    // ccnp/ccp = 00
                    r.set_ccnp(self.ch, false);
                    r.set_ccp(self.ch, false);
                },
                CaptureInputPolarity::FallingEdge => {
                    // ccnp/ccp = 01
                    r.set_ccnp(self.ch, false);
                    r.set_ccp(self.ch, true);
                },
                CaptureInputPolarity::Reserved => {
                    // ccnp/ccp = 10
                    r.set_ccnp(self.ch, true);
                    r.set_ccp(self.ch, false);
                }
                CaptureInputPolarity::BothEdge => {
                    // ccnp/ccp = 11
                    r.set_ccnp(self.ch, true);
                    r.set_ccp(self.ch, true);
                },
            }
        )
    }
    pub fn read_input_polarity(&self) -> CaptureInputPolarity {
        let ccp = self.regs_core().ccer().read().ccp(self.ch);
        let ccnp = self.regs_core().ccer().read().ccp(self.ch);
        match (ccnp, ccp) {
            (false, false) => CaptureInputPolarity::RisingEdge,
            (false, true) => CaptureInputPolarity::FallingEdge,
            (true, false) => CaptureInputPolarity::Reserved,
            (true, true) => CaptureInputPolarity::BothEdge,
        }
    }
    pub fn write_compare_value(&self, val: u16) {
        self.regs_core().ccr(self.ch).write(|r| r.set_ccr(val))
    }
    pub fn read_compare_value(&self) -> u16 {
        self.regs_core().ccr(self.ch).read().ccr()
    }
    pub fn read_capture_value(&self) -> u16 {
        self.read_compare_value()
    }
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PB7, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
