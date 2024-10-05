#![no_std]
#![no_main]

use core::marker::PhantomData;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::pac;

pub enum ChannelAvailable {}
pub enum ChannelAllocated {}

pub trait ChannelState {}
impl ChannelState for ChannelAvailable {}
impl ChannelState for ChannelAllocated {}

pub struct ChannelFactory<Ch1: ChannelState, Ch2: ChannelState, Ch3: ChannelState, Ch4: ChannelState> {
    regs_ptr: *mut(),
    phantom: PhantomData<(Ch1, Ch2, Ch3, Ch4)>,
}

impl<C2: ChannelState, C3: ChannelState, C4: ChannelState> ChannelFactory<ChannelAvailable, C2, C3, C4> {
    pub fn get_capture_ch1(self) -> (ChannelFactory<ChannelAllocated, C2, C3, C4>, TimerCaptureChannel) {
        let this = TimerCaptureChannel {
            regs_ptr: self.regs_ptr,
            channel: TimerChannel::Ch1,
        };
        (
            ChannelFactory {
                regs_ptr: self.regs_ptr,
                phantom: PhantomData
            },
            this
        )
    }
    pub fn get_compare_ch1(self) -> (ChannelFactory<ChannelAllocated, C2, C3, C4>, TimerCompareChannel) {
        let this = TimerCompareChannel {
            regs_ptr: self.regs_ptr,
            channel: TimerChannel::Ch1,
        };
        (
            ChannelFactory {
                regs_ptr: self.regs_ptr,
                phantom: PhantomData
            },
            this
        )
    }
}

impl<C1: ChannelState, C3: ChannelState, C4: ChannelState> ChannelFactory<C1, ChannelAvailable, C3, C4> {
    pub fn get_capture_ch2(self) -> (ChannelFactory<C1, ChannelAllocated, C3, C4>, TimerCaptureChannel) {
        let this = TimerCaptureChannel {
            regs_ptr: self.regs_ptr,
            channel: TimerChannel::Ch2,
        };
        (
            ChannelFactory {
                regs_ptr: self.regs_ptr,
                phantom: PhantomData
            },
            this
        )
    }
    pub fn get_compare_ch2(self) -> (ChannelFactory<C1, ChannelAllocated, C3, C4>, TimerCompareChannel) {
        let this = TimerCompareChannel {
            regs_ptr: self.regs_ptr,
            channel: TimerChannel::Ch2,
        };
        (
            ChannelFactory {
                regs_ptr: self.regs_ptr,
                phantom: PhantomData
            },
            this
        )
    }
}

impl<C1: ChannelState, C2: ChannelState, C4: ChannelState> ChannelFactory<C1, C2, ChannelAvailable, C4> {
    pub fn get_capture_ch3(self) -> (ChannelFactory<C1, C2, ChannelAllocated, C4>, TimerCaptureChannel) {
        let this = TimerCaptureChannel {
            regs_ptr: self.regs_ptr,
            channel: TimerChannel::Ch3,
        };
        (
            ChannelFactory {
                regs_ptr: self.regs_ptr,
                phantom: PhantomData
            },
            this
        )
    }
    pub fn get_compare_ch3(&self) -> (ChannelFactory<C1, C2, ChannelAllocated, C4>, TimerCompareChannel) {
        let this = TimerCompareChannel {
            regs_ptr: self.regs_ptr,
            channel: TimerChannel::Ch3,
        };
        (
            ChannelFactory {
                regs_ptr: self.regs_ptr,
                phantom: PhantomData
            },
            this
        )
    }
}

impl<C1: ChannelState, C2: ChannelState, C3: ChannelState> ChannelFactory<C1, C2, C3, ChannelAvailable> {
    pub fn get_capture_ch4(self) -> (ChannelFactory<C1, C2, C3, ChannelAllocated>, TimerCaptureChannel) {
        let this = TimerCaptureChannel {
            regs_ptr: self.regs_ptr,
            channel: TimerChannel::Ch4,
        };
        (
            ChannelFactory {
                regs_ptr: self.regs_ptr,
                phantom: PhantomData
            },
            this
        )
    }
    pub fn get_compare_ch4(self) -> (ChannelFactory<C1, C2, C3, ChannelAllocated>, TimerCompareChannel) {
        let this = TimerCompareChannel {
            regs_ptr: self.regs_ptr,
            channel: TimerChannel::Ch4,
        };
        (
            ChannelFactory {
                regs_ptr: self.regs_ptr,
                phantom: PhantomData
            },
            this
        )
    }
}

pub struct MyTimer {
    regs_ptr: *mut()
}

impl MyTimer {
    pub fn new(ptr: *mut()) -> Self {
        Self {
            regs_ptr: ptr
        }
    }

    pub fn split(self) -> ChannelFactory<ChannelAvailable, ChannelAvailable, ChannelAvailable, ChannelAvailable> {
        ChannelFactory {
            regs_ptr: self.regs_ptr,
            phantom: PhantomData,
        }
    }
}

impl TimerCore for MyTimer {
    fn regs(&self) -> pac::timer::TimCore {
        unsafe { pac::timer::TimCore::from_ptr(self.regs_ptr) }
    }
}

pub trait TimerCore {
    fn regs(&self) -> pac::timer::TimCore;
    fn write_counter_enable(&self, val: bool) {
        self.regs().cr1().modify(|r| r.set_cen(val));
    }
    fn read_counter_enable(&self) -> bool {
        self.regs().cr1().read().cen()
    }
    fn write_update_disable(&self, val: bool) {
        self.regs().cr1().modify(|r| r.set_udis(val));
    }
    fn read_update_disable(&self) -> bool {
        self.regs().cr1().read().udis()
    }
    fn write_update_request_source(&self, val: pac::timer::vals::Urs) {
        self.regs().cr1().modify(|r| r.set_urs(val));
    }
    fn read_update_request_source(&self) -> pac::timer::vals::Urs {
        self.regs().cr1().read().urs()
    }
    fn write_one_pulse_mode(&self, val: bool) {
        self.regs().cr1().modify(|r| r.set_opm(val));
    }
    fn read_one_pulse_mode(&self) -> bool {
        self.regs().cr1().read().opm()
    }
    fn write_autoreload_preload_enable(&self, val: bool) {
        self.regs().cr1().modify(|r| r.set_arpe(val))
    }
    fn read_autoreload_preload_enable(&self) -> bool {
        self.regs().cr1().read().arpe()
    }
}

pub trait TimerGp16: TimerCore {
    fn regs_gp16(&self) -> pac::timer::TimGp16;
    fn write_direction(&self, val: pac::timer::vals::Dir) {
        self.regs_gp16().cr1().modify(|r| r.set_dir(val))
    }
    fn read_direction(&self) -> pac::timer::vals::Dir {
        self.regs_gp16().cr1().read().dir()
    }
    fn write_centeraligned_mode_selection(&self, val: pac::timer::vals::Cms) {
        self.regs_gp16().cr1().modify(|r| r.set_cms(val))
    }
    fn read_centeraligned_mode_selection(&self) -> pac::timer::vals::Cms {
        self.regs_gp16().cr1().read().cms()
    }
    fn write_clock_division(&self, val: pac::timer::vals::Ckd) {
        self.regs_gp16().cr1().modify(|r| r.set_ckd(val))
    }
    fn read_clock_division(&self) -> pac::timer::vals::Ckd {
        self.regs_gp16().cr1().read().ckd()
    }
}

pub enum CaptureInputPolarity {
    RisingEdge,
    FallingEdge,
    Reserved,
    BothEdge,
}

#[derive(Clone, Copy)]
pub enum TimerChannel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
}

impl Into<usize> for TimerChannel {
    fn into(self) -> usize {
        match self {
            TimerChannel::Ch1=>0,
            TimerChannel::Ch2=>1,
            TimerChannel::Ch3=>2,
            TimerChannel::Ch4=>3,
        }
    }
}


#[derive(Clone, Copy)]
pub struct TimerCaptureChannel {
    regs_ptr: *mut(),
    channel: TimerChannel,
}
impl TimerCaptureCompareBase for TimerCaptureChannel {
    fn regs(&self) -> pac::timer::TimGp16 {
        unsafe { pac::timer::TimGp16::from_ptr(self.regs_ptr) }
    }
    fn ch(&self) -> usize {
        match self.channel {
            TimerChannel::Ch1 => 0,
            TimerChannel::Ch2 => 0,
            TimerChannel::Ch3 => 0,
            TimerChannel::Ch4 => 0,
        }
    }
}
impl TimerCapture for TimerCaptureChannel {}

#[derive(Clone, Copy)]
pub struct TimerCompareChannel {
    regs_ptr: *mut(),
    channel: TimerChannel,
}
impl TimerCaptureCompareBase for TimerCompareChannel {
    fn regs(&self) -> pac::timer::TimGp16 {
        unsafe { pac::timer::TimGp16::from_ptr(self.regs_ptr) }
    }
    fn ch(&self) -> usize {
        match self.channel {
            TimerChannel::Ch1 => 0,
            TimerChannel::Ch2 => 0,
            TimerChannel::Ch3 => 0,
            TimerChannel::Ch4 => 0,
        }
    }
}
impl TimerCompare for TimerCompareChannel {}

pub trait TimerCaptureCompareBase {
    fn regs(&self) -> pac::timer::TimGp16;
    fn ch(&self) -> usize;

    fn write_dma_request_enable(&self, val: bool) {
        self.regs().dier().modify(|r| r.set_ccde(self.ch(), val));
    }
    fn read_dma_request_enable(&self) -> bool {
        self.regs().dier().read().ccde(self.ch())
    }
    fn write_interrupt_enable(&self, val: bool) {
        self.regs().dier().modify(|r| r.set_ccie(self.ch(), val));
    }
    fn read_interrupt_enable(&self) -> bool {
        self.regs().dier().read().ccie(self.ch())
    }
    fn write_interrupt_flag(&self) {
        // different behavior capture compare !
        self.regs().sr().modify(|r| r.set_ccif(self.ch(), false)); // cleared by writing 0
    }
    fn read_interrupt_flag(&self) -> bool {
        // different behavior capture compare !
        self.regs().sr().read().ccif(self.ch())
    }
    fn write_overcapture_flag(&self) {
        self.regs().sr().modify(|r| r.set_ccof(self.ch(), false)); // cleared by writing 0
    }
    fn read_overcapture_flag(&self) -> bool {
        self.regs().sr().read().ccof(self.ch())
    }
    fn write_generation(&self) {
        // different behavior capture compare !
        // egr is write only !
        self.regs().egr().write(|r| r.set_ccg(self.ch(), true));
    }
}

pub trait TimerCapture: TimerCaptureCompareBase {
    fn ccmr_input(&self) -> pac::common::Reg<pac::timer::regs::CcmrInput2ch, pac::common::RW> {
        self.regs().ccmr_input(self.ch() / 2)
    }
    fn write_selection_input(&self, val: pac::timer::vals::CcmrInputCcs) {
        self.ccmr_input().modify(|r| r.set_ccs(self.ch() % 2, val))
    }
    fn read_selection_input(&self) -> pac::timer::vals::CcmrInputCcs {
        self.ccmr_input().read().ccs(self.ch() % 2)
    }
    fn write_prescaler(&self, val: u8) {
        // 0=every event, 1=every 2 events, 2=every 4 events, 3=every 8 events
        self.ccmr_input().modify(|r| r.set_icpsc(self.ch() % 2, val));
    }
    fn read_prescaler(&self) -> u8 {
        self.ccmr_input().read().icpsc(self.ch() % 2)
    }
    fn write_filter(&self, val: pac::timer::vals::FilterValue) {
        self.ccmr_input().modify(|r| r.set_icf(self.ch() % 2, val))
    }
    fn read_filter(&self) -> pac::timer::vals::FilterValue {
        self.ccmr_input().read().icf(self.ch() % 2)
    }
    fn write_capture_enable(&self, val: bool) {
        self.regs().ccer().modify(|r| r.set_cce(self.ch(), val))
    }
    fn read_capture_enable(&self) -> bool {
        self.regs().ccer().read().cce(self.ch())
    }
    fn write_input_polarity(&self, val: CaptureInputPolarity) {
        self.regs().ccer().modify(|r| match val {
                CaptureInputPolarity::RisingEdge => {
                    // ccnp/ccp = 00
                    r.set_ccnp(self.ch(), false);
                    r.set_ccp(self.ch(), false);
                },
                CaptureInputPolarity::FallingEdge => {
                    // ccnp/ccp = 01
                    r.set_ccnp(self.ch(), false);
                    r.set_ccp(self.ch(), true);
                },
                CaptureInputPolarity::Reserved => {
                    // ccnp/ccp = 10
                    r.set_ccnp(self.ch(), true);
                    r.set_ccp(self.ch(), false);
                }
                CaptureInputPolarity::BothEdge => {
                    // ccnp/ccp = 11
                    r.set_ccnp(self.ch(), true);
                    r.set_ccp(self.ch(), true);
                },
            }
        )
    }
    fn read_input_polarity(&self) -> CaptureInputPolarity {
        let ccp = self.regs().ccer().read().ccp(self.ch());
        let ccnp = self.regs().ccer().read().ccp(self.ch());
        match (ccnp, ccp) {
            (false, false) => CaptureInputPolarity::RisingEdge,
            (false, true) => CaptureInputPolarity::FallingEdge,
            (true, false) => CaptureInputPolarity::Reserved,
            (true, true) => CaptureInputPolarity::BothEdge,
        }
    }
    fn write_capture_value(&self, val: u16) {
        self.regs().ccr(self.ch()).write(|r| r.set_ccr(val))
    }
    fn read_capture_value(&self) -> u16 {
        self.regs().ccr(self.ch()).read().ccr()
    }
}

pub trait TimerCompare: TimerCaptureCompareBase {
    fn ccmr_output(&self) -> pac::common::Reg<pac::timer::regs::CcmrOutputGp16, pac::common::RW> {
        self.regs().ccmr_output(self.ch() / 2)
    }
    fn write_selection_output(&self, val: pac::timer::vals::CcmrOutputCcs) {
        self.ccmr_output().modify(|r| r.set_ccs(self.ch() % 2, val))
    }
    fn read_selection_output(&self) -> pac::timer::vals::CcmrOutputCcs {
        self.ccmr_output().read().ccs(self.ch() % 2)
    }
    fn write_fast_enable(&self, val: bool) {
        self.ccmr_output().modify(|r| r.set_ocfe(self.ch() % 2, val))
    }
    fn read_fast_enable(&self) -> bool {
        self.ccmr_output().read().ocfe(self.ch() % 2)
    }
    fn write_preload_enable(&self, val: bool) {
        self.ccmr_output().modify(|r| r.set_ocpe(self.ch() % 2, val));
    }
    fn read_preload_enable(&self) -> bool {
        self.ccmr_output().read().ocpe(self.ch() % 2)
    }
    fn write_mode(&self, val: pac::timer::vals::Ocm) {
        self.ccmr_output().modify(|r| r.set_ocm(self.ch() % 2, val));
    }
    fn read_mode(&self) -> pac::timer::vals::Ocm {
        self.ccmr_output().read().ocm(self.ch() % 2)
    }
    fn write_clear_enable(&self, val: bool) {
        self.ccmr_output().modify(|r| r.set_occe(self.ch() % 2, val))
    }
    fn read_clear_enable(&self) -> bool {
        self.ccmr_output().read().occe(self.ch() % 2)
    }
    fn write_output_enable(&self, val: bool) {
        self.regs().ccer().modify(|r| r.set_cce(self.ch(), val))
    }
    fn read_output_enable(&self) -> bool {
        self.regs().ccer().read().cce(self.ch())
    }
    fn write_output_polarity(&self, val: bool) {
        self.regs().ccer().modify(|r| r.set_ccp(self.ch(), val));
    }
    fn read_output_polarity(&self) -> bool {
        self.regs().ccer().read().ccp(self.ch())
    }
    fn write_compare_value(&self, val: u16) {
        self.regs().ccr(self.ch()).write(|r| r.set_ccr(val))
    }
    fn read_compare_value(&self) -> u16 {
        self.regs().ccr(self.ch()).read().ccr()
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
