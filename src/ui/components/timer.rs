//! Countdown display. The owner retains this entity across page changes.
use crate::ui::theme;
use gpui::{Context, EventEmitter, Render, SharedString, Task, Window, div, prelude::*, px, rgb};
use std::time::{Duration, SystemTime};

#[derive(Clone, Copy)]
pub(crate) enum Deadline {
    Elapsed {
        started: Duration,
        duration: Duration,
    },
    Calendar(SystemTime),
}
/// CLOCK_BOOTTIME includes suspend on Linux. GetTickCount64 does on Windows.
/// std::time::Instant does not have the same suspend contract on both platforms.
fn elapsed_clock() -> Duration {
    #[cfg(target_os = "linux")]
    {
        let mut time = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: a valid timespec pointer and a supported monotonic clock ID.
        if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut time) } == 0 {
            return Duration::new(time.tv_sec as u64, time.tv_nsec as u32);
        }
        panic!("CLOCK_BOOTTIME unavailable");
    }
    #[cfg(windows)]
    {
        Duration::from_millis(unsafe {
            windows_sys::Win32::System::SystemInformation::GetTickCount64()
        })
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        compile_error!("Timer requires a suspend-aware elapsed clock for this platform");
    }
}
impl Deadline {
    pub fn duration(duration: Duration) -> Self {
        Self::Elapsed {
            started: elapsed_clock(),
            duration,
        }
    }
    fn remaining(self) -> Duration {
        self.remaining_at(elapsed_clock(), SystemTime::now())
    }
    fn remaining_at(self, elapsed: Duration, calendar: SystemTime) -> Duration {
        match self {
            Self::Elapsed { started, duration } => {
                duration.saturating_sub(elapsed.saturating_sub(started))
            }
            Self::Calendar(end) => end.duration_since(calendar).unwrap_or_default(),
        }
    }
}
pub(crate) struct Expired;
pub(crate) struct Timer {
    deadline: Deadline,
    emitted: bool,
    text: SharedString,
    max_units: usize,
    tick: Option<Task<()>>,
}
impl EventEmitter<Expired> for Timer {}
impl Timer {
    pub fn new(deadline: Deadline) -> Self {
        Self {
            deadline,
            emitted: false,
            text: "Disponible".into(),
            max_units: 3,
            tick: None,
        }
    }
    pub fn set_max_units(
        &mut self,
        max_units: usize,
        cx: &mut Context<Self>,
    ) -> super::state::Validation<()> {
        if !(1..=3).contains(&max_units) {
            return Err("Le Timer affiche entre une et trois unités".into());
        }
        self.max_units = max_units;
        cx.notify();
        Ok(())
    }
    pub fn restart(&mut self, deadline: Deadline, cx: &mut Context<Self>) {
        self.deadline = deadline;
        self.emitted = false;
        self.tick = None;
        cx.notify();
    }
    pub fn expired_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.text = text.into();
        cx.notify();
    }
}
pub(crate) fn format_remaining(remaining: Duration, max_units: usize) -> Vec<(u64, &'static str)> {
    let seconds = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
    let values = [
        (seconds / 86400, "d"),
        ((seconds / 3600) % 24, "h"),
        ((seconds / 60) % 60, "m"),
        (seconds % 60, "s"),
    ];
    let start = values.iter().position(|(n, _)| *n > 0).unwrap_or(3);
    values
        .into_iter()
        .skip(start)
        .take(max_units.clamp(1, 3))
        .collect()
}
impl Render for Timer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let remaining = if self.emitted {
            Duration::ZERO
        } else {
            self.deadline.remaining()
        };
        if remaining.is_zero() && !self.emitted {
            self.emitted = true;
            cx.emit(Expired);
            self.tick = None;
        }
        if !remaining.is_zero() && self.tick.is_none() {
            self.tick = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(Duration::from_secs(1)).await;
                    let Ok(expired) = this.update(cx, |this, cx| {
                        cx.notify();
                        this.deadline.remaining().is_zero()
                    }) else {
                        break;
                    };
                    if expired {
                        break;
                    }
                }
            }));
        }
        div()
            .flex()
            .items_center()
            .gap_1()
            .font_family(theme::FONT_FAMILY)
            .text_size(px(theme::FONT_SIZE))
            .line_height(px(theme::controls::LINE_HEIGHT))
            .font_weight(gpui::FontWeight(theme::tokens::TOKEN_FONT_MEDIUM))
            .when(remaining.is_zero(), |d| {
                d.text_color(rgb(theme::SUCCESS))
                    .child(div().text_color(rgb(theme::SUCCESS)).child("●"))
                    .child(self.text.clone())
            })
            .when(!remaining.is_zero(), |d| {
                d.children(format_remaining(remaining, self.max_units).into_iter().map(
                    |(n, unit)| {
                        div()
                            .flex()
                            .items_baseline()
                            .gap_1()
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight(theme::tokens::TOKEN_FONT_MEDIUM))
                                    .text_color(rgb(theme::TIMER_TEXT))
                                    .child(n.to_string()),
                            )
                            .child(
                                div()
                                    .text_size(px(theme::tokens::TOKEN_XS))
                                    .text_color(rgb(theme::TIMER_UNIT))
                                    .child(unit),
                            )
                    },
                ))
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn countdown_uses_elapsed_time_including_suspend_not_wall_clock() {
        let deadline = Deadline::Elapsed {
            started: Duration::from_secs(100),
            duration: Duration::from_secs(120),
        };
        assert_eq!(
            deadline.remaining_at(Duration::from_secs(190), SystemTime::UNIX_EPOCH),
            Duration::from_secs(30)
        );
        assert_eq!(
            deadline.remaining_at(Duration::from_secs(230), SystemTime::now()),
            Duration::ZERO
        );
    }
    #[test]
    fn calendar_deadline_tracks_wall_clock() {
        let deadline = Deadline::Calendar(SystemTime::UNIX_EPOCH + Duration::from_secs(100));
        assert_eq!(
            deadline.remaining_at(
                Duration::ZERO,
                SystemTime::UNIX_EPOCH + Duration::from_secs(40)
            ),
            Duration::from_secs(60)
        );
    }
    #[test]
    fn formats_zero_units_and_rounds_up() {
        assert_eq!(
            format_remaining(Duration::from_secs(86400 + 300), 3),
            vec![(1, "d"), (0, "h"), (5, "m")]
        );
        assert_eq!(
            format_remaining(Duration::from_millis(60001), 3),
            vec![(1, "m"), (1, "s")]
        );
        assert_eq!(
            format_remaining(Duration::from_secs(3605), 2),
            vec![(1, "h"), (0, "m")]
        );
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::TestAppContext;
    use std::{cell::Cell, rc::Rc};
    #[gpui::test]
    fn expiration_is_unique_until_explicit_restart(cx: &mut TestAppContext) {
        let (timer, cx) = cx.add_window_view(|_, _| {
            Timer::new(Deadline::Calendar(
                SystemTime::now() + Duration::from_secs(600),
            ))
        });
        let count = Rc::new(Cell::new(0));
        let output = count.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&timer, move |_, _: &Expired, _| {
                output.set(output.get() + 1)
            })
        });
        cx.update(|_, cx| {
            timer.update(cx, |timer, cx| {
                timer.restart(Deadline::Calendar(SystemTime::UNIX_EPOCH), cx)
            })
        });
        for _ in 0..3 {
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.run_until_parked();
        }
        assert_eq!(count.get(), 1);
        // Simulate a backwards wall-clock correction after expiration: the
        // completed countdown must stay expired until explicit restart.
        cx.update(|_, cx| {
            timer.update(cx, |timer, cx| {
                timer.deadline = Deadline::Calendar(SystemTime::now() + Duration::from_secs(600));
                cx.notify();
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| assert!(timer.read(cx).tick.is_none()));
        cx.update(|_, cx| {
            timer.update(cx, |timer, cx| {
                assert!(timer.set_max_units(0, cx).is_err());
                assert_eq!(timer.max_units, 3);
                timer.restart(Deadline::Calendar(SystemTime::UNIX_EPOCH), cx);
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        assert_eq!(count.get(), 2);
    }
}
