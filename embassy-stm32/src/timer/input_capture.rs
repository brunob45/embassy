//! Input capture driver.

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy_hal_internal::{into_ref, PeripheralRef};

use super::low_level::{CountingMode, FilterValue, InputCaptureMode, InputTISelection, Timer};
use super::{
    InterruptHandler, Channel, Channel1Pin, Channel2Pin, Channel3Pin, Channel4Pin,
    GeneralInstance4Channel,
};
use crate::gpio::{AFType, AnyPin, Pull};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::time::Hertz;
use crate::Peripheral;

/// Channel 1 marker type.
pub enum Ch1 {}
/// Channel 2 marker type.
pub enum Ch2 {}
/// Channel 3 marker type.
pub enum Ch3 {}
/// Channel 4 marker type.
pub enum Ch4 {}

/// Capture pin wrapper.
///
/// This wraps a pin to make it usable with capture.
pub struct CapturePin<'d, T, C> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, T: GeneralInstance4Channel> CapturePin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " capture pin instance.")]
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<T>> + 'd, pull_type: Pull) -> Self {
                into_ref!(pin);

                pin.set_as_af_pull(pin.af_num(), AFType::Input, pull_type);

                CapturePin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, Ch1, Channel1Pin);
channel_impl!(new_ch2, Ch2, Channel2Pin);
channel_impl!(new_ch3, Ch3, Channel3Pin);
channel_impl!(new_ch4, Ch4, Channel4Pin);

/// Input capture driver.
pub struct InputCapture<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> InputCapture<'d, T> {
    /// Create a new input capture driver.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1: Option<CapturePin<'d, T, Ch1>>,
        _ch2: Option<CapturePin<'d, T, Ch2>>,
        _ch3: Option<CapturePin<'d, T, Ch3>>,
        _ch4: Option<CapturePin<'d, T, Ch4>>,
        _irq: impl Binding<T::UpdateInterrupt, InterruptHandler<T>> + 'd,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd, freq: Hertz, counting_mode: CountingMode) -> Self {
        let mut inner = Timer::new(tim);

        inner.set_counting_mode(counting_mode);
        inner.set_tick_freq(freq);
        inner.enable_outputs(); // Required for advanced timers, see GeneralInstance4Channel for details
        inner.start();

        // enable NVIC interrupt
        T::CaptureCompareInterrupt::unpend();
        unsafe { T::CaptureCompareInterrupt::enable() };

        Self { inner }
    }

    /// Enable the given channel.
    pub fn enable(&mut self, channel: Channel) {
        self.inner.enable_channel(channel, true);
    }

    /// Disable the given channel.
    pub fn disable(&mut self, channel: Channel) {
        self.inner.enable_channel(channel, false);
    }

    /// Check whether given channel is enabled
    pub fn is_enabled(&self, channel: Channel) -> bool {
        self.inner.get_channel_enable_state(channel)
    }

    /// Set the input capture mode for a given channel.
    pub fn set_input_capture_mode(&mut self, channel: Channel, mode: InputCaptureMode) {
        self.inner.set_input_capture_mode(channel, mode);
    }

    /// Set input TI selection.
    pub fn set_input_ti_selection(&mut self, channel: Channel, tisel: InputTISelection) {
        self.inner.set_input_ti_selection(channel, tisel)
    }

    /// Get capture value for a channel.
    pub fn get_capture_value(&self, channel: Channel) -> u32 {
        self.inner.get_capture_value(channel)
    }

    /// Get input interrupt.
    pub fn get_input_interrupt(&self, channel: Channel) -> bool {
        self.inner.get_input_interrupt(channel)
    }

    fn new_future(&self, channel: Channel, mode: InputCaptureMode, tisel: InputTISelection) -> InputCaptureFuture<T> {
        // Configuration steps from ST RM0390 (STM32F446) chapter 17.3.5
        // or ST RM0008 (STM32F103) chapter 15.3.5 Input capture mode
        self.inner.set_input_ti_selection(channel, tisel);
        self.inner.set_input_capture_filter(channel, FilterValue::NOFILTER);
        self.inner.set_input_capture_mode(channel, mode);
        self.inner.set_input_capture_prescaler(channel, 0);
        self.inner.enable_channel(channel, true);

        InputCaptureFuture::new(InterruptFlag::from(channel))
    }

    /// Asynchronously wait until the pin sees a rising edge.
    pub async fn wait_for_rising_edge(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::Rising, InputTISelection::Normal)
            .await
    }

    /// Asynchronously wait until the pin sees a falling edge.
    pub async fn wait_for_falling_edge(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::Falling, InputTISelection::Normal)
            .await
    }

    /// Asynchronously wait until the pin sees any edge.
    pub async fn wait_for_any_edge(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::BothEdges, InputTISelection::Normal)
            .await
    }

    /// Asynchronously wait until the (alternate) pin sees a rising edge.
    pub async fn wait_for_rising_edge_alternate(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::Rising, InputTISelection::Alternate)
            .await
    }

    /// Asynchronously wait until the (alternate) pin sees a falling edge.
    pub async fn wait_for_falling_edge_alternate(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::Falling, InputTISelection::Alternate)
            .await
    }

    /// Asynchronously wait until the (alternate) pin sees any edge.
    pub async fn wait_for_any_edge_alternate(&mut self, channel: Channel) -> u32 {
        self.new_future(channel, InputCaptureMode::BothEdges, InputTISelection::Alternate)
            .await
    }
}

/// All the timer interrupt flags
#[derive(Clone, Copy)]
pub enum InterruptFlag {
    /// Update interrupt
    Update,
    /// Capture/Compare 1 interrupt
    CaptureCompare1,
    /// Capture/Compare 2 interrupt
    CaptureCompare2,
    /// Capture/Compare 3 interrupt
    CaptureCompare3,
    /// Capture/Compare 4 interrupt
    CaptureCompare4,
    /// COM interrupt
    ComEvent,
    /// Trigger interrupt
    Trigger,
    /// Break interrupt
    Break,
}

impl From<InterruptFlag> for u32 {
    fn from(value: InterruptFlag) -> Self {
        match value {
            InterruptFlag::Update => 1 << 0,
            InterruptFlag::CaptureCompare1 => 1 << 1,
            InterruptFlag::CaptureCompare2 => 1 << 2,
            InterruptFlag::CaptureCompare3 => 1 << 3,
            InterruptFlag::CaptureCompare4 => 1 << 4,
            InterruptFlag::ComEvent => 1 << 5,
            InterruptFlag::Trigger => 1 << 6,
            InterruptFlag::Break => 1 << 7,
        }
    }
}

impl From<Channel> for InterruptFlag {
    fn from(value: Channel) -> Self {
        match value {
            Channel::Ch1 => InterruptFlag::CaptureCompare1,
            Channel::Ch2 => InterruptFlag::CaptureCompare2,
            Channel::Ch3 => InterruptFlag::CaptureCompare3,
            Channel::Ch4 => InterruptFlag::CaptureCompare4,
        }
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputCaptureFuture<T: GeneralInstance4Channel> {
    flag: InterruptFlag,
    phantom: PhantomData<T>,
}

impl<T: GeneralInstance4Channel> InputCaptureFuture<T> {
    pub fn new(flag: InterruptFlag) -> Self {
        critical_section::with(|_| {
            let regs = unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) };

            // disable interrupt enable
            regs.dier().modify(|w| w.0 |= u32::from(flag));
        });

        Self {
            flag,
            phantom: PhantomData,
        }
    }
}

impl<T: GeneralInstance4Channel> Drop for InputCaptureFuture<T> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            let regs = unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) };

            // disable interrupt enable
            regs.dier().modify(|w| w.0 &= !u32::from(self.flag));
        });
    }
}

impl<T: GeneralInstance4Channel> Future for InputCaptureFuture<T> {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        T::state().wakers[u32::from(self.flag) as usize].register(cx.waker());

        let regs = unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) };
        let dier = regs.dier().read();
        let enabled = (dier.0 & u32::from(self.flag)) != 0;
        if !enabled {
            let val = match self.flag {
                InterruptFlag::CaptureCompare1 => regs.ccr(1).read().0,
                InterruptFlag::CaptureCompare2 => regs.ccr(2).read().0,
                InterruptFlag::CaptureCompare3 => regs.ccr(3).read().0,
                InterruptFlag::CaptureCompare4 => regs.ccr(4).read().0,
                _ => regs.cnt().read().0,
            };
            Poll::Ready(val)
        } else {
            Poll::Pending
        }
    }
}
