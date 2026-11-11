//! Export du bilan : copie, fichier texte et image PNG dessinée dans un canvas.

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlAnchorElement, HtmlCanvasElement};

use super::{document, window};
use crate::habits::{Cell, Day, Habit, MONTHS};

pub fn copy(text: &str) {
    let clipboard = window().navigator().clipboard();
    let _ = clipboard.write_text(text);
}

fn click_download(href: &str, file: &str) {
    if let Ok(a) = document().create_element("a") {
        let a: HtmlAnchorElement = a.unchecked_into();
        a.set_href(href);
        a.set_download(file);
        a.click();
    }
}

pub fn download_text(text: &str, file: &str) {
    let parts = js_sys::Array::of1(&JsValue::from_str(text));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/plain;charset=utf-8");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options) else {
        return;
    };
    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
        click_download(&url, file);
        super::later(1000, move || {
            let _ = web_sys::Url::revoke_object_url(&url);
        });
    }
}

fn round_rect(ctx: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, r: f64) {
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.arc_to(x + w, y, x + w, y + h, r).ok();
    ctx.arc_to(x + w, y + h, x, y + h, r).ok();
    ctx.arc_to(x, y + h, x, y, r).ok();
    ctx.arc_to(x, y, x + w, y, r).ok();
    ctx.close_path();
}

/// Une carte par habitude : son nom, sa série et ses 5 dernières semaines.
pub fn download_image(habits: &[Habit], today: Day) {
    let Ok(canvas) = document().create_element("canvas") else {
        return;
    };
    let canvas: HtmlCanvasElement = canvas.unchecked_into();
    let width = 1080.0;
    let row = 300.0;
    let height = 260.0 + row * habits.len().max(1) as f64 + 80.0;
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);
    let Some(ctx) = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
    else {
        return;
    };

    // Le fond : un dégradé nuit, comme le site.
    let bg = ctx.create_linear_gradient(0.0, 0.0, width, height);
    let _ = bg.add_color_stop(0.0, "#0b0820");
    let _ = bg.add_color_stop(1.0, "#140b2e");
    ctx.set_fill_style_canvas_gradient(&bg);
    ctx.fill_rect(0.0, 0.0, width, height);

    let (y, m, d) = today.ymd();
    ctx.set_fill_style_str("#ffffff");
    ctx.set_font("800 64px Syne, system-ui, sans-serif");
    let _ = ctx.fill_text("Routia", 72.0, 130.0);
    ctx.set_fill_style_str("#a5a0c8");
    ctx.set_font("500 30px Inter, system-ui, sans-serif");
    let _ = ctx.fill_text(
        &format!("Bilan du {d} {} {y}", MONTHS[(m - 1) as usize]),
        72.0,
        182.0,
    );

    if habits.is_empty() {
        ctx.set_fill_style_str("#ffffff");
        let _ = ctx.fill_text("Aucune habitude pour l'instant.", 72.0, 300.0);
    }

    let first = today.week_start().shift(-28);
    for (i, h) in habits.iter().enumerate() {
        let top = 240.0 + i as f64 * row;
        ctx.set_fill_style_str("rgba(255,255,255,0.06)");
        round_rect(&ctx, 56.0, top, width - 112.0, row - 32.0, 32.0);
        ctx.fill();
        ctx.set_fill_style_str(h.color());
        round_rect(&ctx, 56.0, top, 12.0, row - 32.0, 6.0);
        ctx.fill();

        let streak = h.streak(today);
        ctx.set_fill_style_str("#ffffff");
        ctx.set_font("700 40px Inter, system-ui, sans-serif");
        let _ = ctx.fill_text(&h.name, 100.0, top + 70.0);
        ctx.set_fill_style_str("#a5a0c8");
        ctx.set_font("500 26px Inter, system-ui, sans-serif");
        let _ = ctx.fill_text(
            &format!(
                "{} · record {} · {} % sur 30 j",
                h.frequency_label(),
                h.best_streak(today),
                h.rate(today, 30)
            ),
            100.0,
            top + 112.0,
        );
        ctx.set_fill_style_str(h.color());
        ctx.set_font("800 72px Syne, system-ui, sans-serif");
        ctx.set_text_align("right");
        let _ = ctx.fill_text(&format!("🔥 {streak}"), width - 100.0, top + 92.0);
        ctx.set_font("500 24px Inter, system-ui, sans-serif");
        ctx.set_fill_style_str("#a5a0c8");
        let _ = ctx.fill_text(h.unit().label(streak), width - 100.0, top + 126.0);
        ctx.set_text_align("left");

        // 35 cases, en une ligne.
        let size = 20.0;
        let gap = 4.0;
        for k in 0..35 {
            let day = first.shift(k);
            let x = 100.0 + f64::from(k) * (size + gap);
            let cy = top + 170.0;
            let color = match h.cell(day, today) {
                Cell::Done => h.color(),
                Cell::Missed => "rgba(251,113,133,0.55)",
                Cell::Pending => "rgba(255,255,255,0.35)",
                Cell::Rest => "rgba(255,255,255,0.12)",
                Cell::Empty => "rgba(255,255,255,0.04)",
            };
            ctx.set_fill_style_str(color);
            round_rect(&ctx, x, cy, size, size, 6.0);
            ctx.fill();
        }
    }

    // La légende, avec les vraies couleurs.
    ctx.set_font("500 22px Inter, system-ui, sans-serif");
    let mut x = 72.0;
    for (color, label) in [
        ("#7c5cff", "fait"),
        ("rgba(251,113,133,0.55)", "manqué"),
        ("rgba(255,255,255,0.12)", "repos"),
    ] {
        ctx.set_fill_style_str(color);
        round_rect(&ctx, x, height - 66.0, 20.0, 20.0, 5.0);
        ctx.fill();
        ctx.set_fill_style_str("#a5a0c8");
        let _ = ctx.fill_text(label, x + 30.0, height - 49.0);
        x += 30.0 + ctx.measure_text(label).map_or(60.0, |m| m.width()) + 28.0;
    }
    ctx.set_fill_style_str("#6f6a92");
    ctx.set_text_align("right");
    let _ = ctx.fill_text("routia", width - 72.0, height - 49.0);

    if let Ok(url) = canvas.to_data_url_with_type("image/png") {
        click_download(&url, "routia-bilan.png");
    }
}
