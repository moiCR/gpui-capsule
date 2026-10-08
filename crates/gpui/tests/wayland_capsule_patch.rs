#![cfg(all(target_os = "linux", feature = "wayland"))]

use std::time::Duration;

use gpui::{
    App, Context, Render, Window, WindowKind, WindowOptions, div,
    layer_shell::{KeyboardInteractivity, LayerShellOptions},
    prelude::*,
    rgb,
    session_lock::SessionLockOptions,
};

struct LockView;

impl Render for LockView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(rgb(0x202020))
    }
}

#[test]
#[ignore = "Requires GPUI_CAPSULE_PATCH_TEST=1 and an isolated Wayland compositor with two outputs"]
fn wayland_capsule_patch_lifecycle() {
    assert_eq!(std::env::var("GPUI_CAPSULE_PATCH_TEST").as_deref(), Ok("1"));
    let scenario = std::env::var("GPUI_CAPSULE_PATCH_SCENARIO").unwrap_or_else(|_| "lock".into());
    gpui_platform::application().run(move |cx: &mut App| {
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;
            cx.update(|cx| run_scenario(cx, scenario));
        })
        .detach();
    });
}

fn run_scenario(cx: &mut App, scenario: String) {
    let displays = cx.displays();
    assert!(displays.len() >= 2);
    let keeper = cx
        .open_window(WindowOptions::default(), |_, cx| cx.new(|_| LockView))
        .unwrap();
    if scenario == "layer" {
        let layer = cx
            .open_window(
                WindowOptions::new().kind(WindowKind::LayerShell(LayerShellOptions::default())),
                |window, cx| {
                    for _ in 0..3 {
                        window.set_keyboard_interactivity(KeyboardInteractivity::None);
                    }
                    for _ in 0..3 {
                        window.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
                    }
                    cx.new(|_| LockView)
                },
            )
            .unwrap();
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(600))
                .await;
            layer
                .update(cx, |_, window, _| window.remove_window())
                .unwrap();
            keeper
                .update(cx, |_, window, _| window.remove_window())
                .unwrap();
            cx.update(|cx| cx.quit());
        })
        .detach();
        return;
    }
    let options = |display| {
        WindowOptions::new()
            .display_id(Some(display))
            .kind(WindowKind::SessionLock(SessionLockOptions {}))
    };
    let first = cx
        .open_window(options(displays[0].id()), |window, cx| {
            window.set_input_region(None);
            cx.new(|_| LockView)
        })
        .unwrap();
    let duplicate = cx.open_window(options(displays[0].id()), |_, cx| cx.new(|_| LockView));
    assert!(duplicate.is_err());
    let second = cx
        .open_window(options(displays[1].id()), |window, cx| {
            window.set_input_region(None);
            cx.new(|_| LockView)
        })
        .unwrap();
    if scenario == "pending" {
        first
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        second
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
    }
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(Duration::from_millis(1000))
            .await;
        if scenario == "lock" || scenario == "hold" {
            first
                .update(cx, |_, window, _| window.remove_window())
                .unwrap();
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;
            if scenario == "hold" {
                cx.background_executor().timer(Duration::from_secs(4)).await;
            }
            second
                .update(cx, |_, window, _| window.remove_window())
                .unwrap();
        } else if scenario == "rejected" {
            assert!(first.update(cx, |_, _, _| ()).is_err());
            assert!(second.update(cx, |_, _, _| ()).is_err());
        }
        cx.background_executor()
            .timer(Duration::from_millis(500))
            .await;
        if scenario == "rejected" {
            keeper
                .update(cx, |_, window, _| window.remove_window())
                .unwrap();
            cx.update(|cx| cx.quit());
            return;
        }
        let reopened = cx
            .update(|cx| cx.open_window(options(displays[0].id()), |_, cx| cx.new(|_| LockView)))
            .unwrap();
        cx.background_executor()
            .timer(Duration::from_millis(500))
            .await;
        reopened
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        keeper
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        cx.update(|cx| cx.quit());
    })
    .detach();
}
