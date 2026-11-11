//! Les animations 2D dessinées dans un canvas (API Canvas 2D, en Rust) : les anneaux
//! d'habitudes de l'accueil et la tour des jours.

use std::cell::{Cell, RefCell};
use std::f64::consts::{PI, TAU};
use std::rc::Rc;

use paquets::prelude::State;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, IntersectionObserver, IntersectionObserverEntry,
};

use super::{on_passive, reduced_motion, scroll_y, viewport, window};

// ----- Outils -----------------------------------------------------------------------

/// Un canvas et son contexte 2D, à la taille que lui donne le CSS (× la densité de
/// pixels de l'écran, 2 au plus).
struct Surface {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    w: f64,
    h: f64,
}

impl Surface {
    fn new(el: &web_sys::Element) -> Option<Surface> {
        let canvas: HtmlCanvasElement = el.clone().dyn_into().ok()?;
        let ctx = canvas
            .get_context("2d")
            .ok()
            .flatten()?
            .dyn_into::<CanvasRenderingContext2d>()
            .ok()?;
        Some(Surface {
            canvas,
            ctx,
            w: 0.0,
            h: 0.0,
        })
    }

    /// Suit la taille CSS du canvas ; rend `false` s'il n'a pas de taille.
    fn fit(&mut self) -> bool {
        let w = f64::from(self.canvas.client_width());
        let h = f64::from(self.canvas.client_height());
        if w < 1.0 || h < 1.0 {
            return false;
        }
        let dpr = window().device_pixel_ratio().clamp(1.0, 2.0);
        let (pw, ph) = ((w * dpr).round() as u32, (h * dpr).round() as u32);
        if self.canvas.width() != pw || self.canvas.height() != ph {
            self.canvas.set_width(pw);
            self.canvas.set_height(ph);
        }
        let _ = self.ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
        self.w = w;
        self.h = h;
        self.ctx.clear_rect(0.0, 0.0, w, h);
        true
    }
}

/// Une boucle `requestAnimationFrame` qui s'arrête quand la valeur rendue est libérée.
struct Looping(Rc<Cell<bool>>);

impl Drop for Looping {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

/// La closure de la boucle, gardée pour se reprogrammer à chaque image.
type Slot = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

fn animation_loop(mut frame: impl FnMut(f64) + 'static) -> Looping {
    let running = Rc::new(Cell::new(true));
    let slot: Slot = Rc::new(RefCell::new(None));
    let again = slot.clone();
    let alive = running.clone();
    *slot.borrow_mut() = Some(Closure::new(move |time: f64| {
        if !alive.get() {
            // Libère la closure : la boucle est finie.
            again.borrow_mut().take();
            return;
        }
        frame(time);
        if let Some(cb) = again.borrow().as_ref() {
            let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }));
    if let Some(cb) = slot.borrow().as_ref() {
        let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
    }
    Looping(running)
}

/// Vrai tant que l'élément est à l'écran : une animation cachée ne dessine rien.
fn visibility(el: &web_sys::Element) -> Rc<Cell<bool>> {
    let visible = Rc::new(Cell::new(true));
    let flag = visible.clone();
    let callback = Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
        for entry in entries.iter() {
            if let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() {
                flag.set(entry.is_intersecting());
            }
        }
    });
    if let Ok(io) = IntersectionObserver::new(callback.as_ref().unchecked_ref()) {
        io.observe(el);
    }
    callback.forget();
    visible
}

/// Un nombre pseudo-aléatoire stable, de 0 à 1.
fn hash(i: u32) -> f64 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    f64::from(x % 10_000) / 10_000.0
}

const VIOLET: &str = "#7c5cff";
const CYAN: &str = "#22d3ee";
const PINK: &str = "#f472b6";
const AMBER: &str = "#fbbf24";
const MINT: &str = "#34d399";
const PALETTE: [&str; 5] = [VIOLET, CYAN, MINT, AMBER, PINK];

fn ease_out(x: f64) -> f64 {
    1.0 - (1.0 - x.clamp(0.0, 1.0)).powi(3)
}

// ----- Anneaux d'habitudes ----------------------------------------------------------

/// Une forme qui flotte derrière les anneaux.
struct Shape {
    x: f64,
    y: f64,
    size: f64,
    speed: f64,
    spin: f64,
    depth: f64,
    kind: u32,
    color: &'static str,
}

fn draw_shape(ctx: &CanvasRenderingContext2d, s: &Shape, angle: f64) {
    ctx.save();
    let _ = ctx.translate(s.x, s.y);
    let _ = ctx.rotate(angle);
    ctx.set_fill_style_str(s.color);
    ctx.set_stroke_style_str(s.color);
    ctx.set_line_width(2.0);
    let r = s.size;
    ctx.begin_path();
    match s.kind {
        0 => {
            let _ = ctx.arc(0.0, 0.0, r * 0.55, 0.0, TAU);
            ctx.fill();
        }
        1 => {
            ctx.round_rect_with_f64(-r * 0.5, -r * 0.5, r, r, r * 0.22)
                .ok();
            ctx.fill();
        }
        2 => {
            ctx.move_to(0.0, -r * 0.6);
            ctx.line_to(r * 0.55, r * 0.45);
            ctx.line_to(-r * 0.55, r * 0.45);
            ctx.close_path();
            ctx.fill();
        }
        _ => {
            // Une coche : le geste du jour.
            ctx.set_line_width(r * 0.22);
            ctx.set_line_cap("round");
            ctx.set_line_join("round");
            ctx.move_to(-r * 0.45, 0.0);
            ctx.line_to(-r * 0.1, r * 0.35);
            ctx.line_to(r * 0.5, -r * 0.4);
            ctx.stroke();
        }
    }
    ctx.restore();
}

/// Trois anneaux qui se remplissent comme trois habitudes tenues, un noyau qui bat,
/// sept jours en orbite (cinq faits) et des formes qui flottent. Le pointeur décale les
/// plans (parallaxe), le défilement fait monter la scène.
pub fn hero(canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    let Some(mut surface) = Surface::new(canvas) else {
        return Box::new(|| {}) as Box<dyn FnOnce()>;
    };
    let pointer = Rc::new(Cell::new((0.0f64, 0.0f64)));
    {
        let pointer = pointer.clone();
        on_passive::<web_sys::PointerEvent>(&window(), "pointermove", move |e| {
            let (w, h) = viewport();
            pointer.set((
                f64::from(e.client_x()) / w * 2.0 - 1.0,
                f64::from(e.client_y()) / h * 2.0 - 1.0,
            ));
        });
    }
    let mut shapes: Vec<Shape> = (0..26)
        .map(|i| Shape {
            x: hash(i),
            y: hash(i + 100),
            size: 8.0 + hash(i + 200) * 16.0,
            speed: 0.012 + hash(i + 300) * 0.03,
            spin: (hash(i + 400) - 0.5) * 1.6,
            depth: 0.3 + hash(i + 500) * 0.9,
            kind: i % 4,
            color: [VIOLET, CYAN, PINK, AMBER][(i % 4) as usize],
        })
        .collect();
    let visible = visibility(canvas);
    let calm = reduced_motion();
    let mut tilt = (0.0, 0.0);
    let mut last = 0.0;

    let looping = animation_loop(move |time| {
        if !visible.get() || !surface.fit() {
            return;
        }
        let dt = ((time - last) / 1000.0).clamp(0.0, 0.05);
        last = time;
        let t = if calm { 2.0 } else { time / 1000.0 };
        let (w, h) = (surface.w, surface.h);
        let ctx = &surface.ctx;
        let (px, py) = pointer.get();
        tilt.0 += (px - tilt.0) * 0.06;
        tilt.1 += (py - tilt.1) * 0.06;
        let (_, vh) = viewport();
        let scroll = scroll_y() / vh.max(1.0);

        // Les formes qui flottent, en coordonnées relatives (0 à 1), qui remontent.
        for (i, s) in shapes.iter_mut().enumerate() {
            if !calm {
                s.y -= s.speed * dt;
                if s.y < -0.08 {
                    s.y = 1.08;
                    s.x = hash(i as u32 + (time as u32 % 997));
                }
            }
            let shape = Shape {
                x: s.x * w + tilt.0 * 30.0 * s.depth,
                y: s.y * h + tilt.1 * 20.0 * s.depth - scroll * 120.0 * s.depth,
                size: s.size * s.depth,
                ..*s
            };
            ctx.set_global_alpha(0.18 + 0.35 * s.depth.min(1.0));
            draw_shape(ctx, &shape, t * s.spin + i as f64);
        }
        ctx.set_global_alpha(1.0);

        // Les anneaux.
        let size = w.min(h);
        let cx = w / 2.0 + tilt.0 * 14.0;
        let cy = h / 2.0 + tilt.1 * 10.0 - scroll * h * 0.25;
        let base = size * 0.3;
        let rings = [
            (1.0, VIOLET, CYAN, 0.0),
            (0.76, CYAN, MINT, 1.3),
            (0.52, PINK, AMBER, 2.6),
        ];
        let thickness = (size * 0.055).clamp(10.0, 26.0);
        for (k, (scale, from, to, phase)) in rings.iter().enumerate() {
            let r = base * scale;
            // La piste.
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, r, 0.0, TAU);
            ctx.set_stroke_style_str("rgba(255,255,255,0.07)");
            ctx.set_line_width(thickness);
            ctx.stroke();
            // Le remplissage : il monte, repart, comme une semaine qui avance.
            let cycle = (t * 0.22 + phase / TAU).fract();
            let fill = if cycle < 0.7 {
                ease_out(cycle / 0.7)
            } else {
                1.0
            };
            let alpha = if cycle < 0.85 {
                1.0
            } else {
                1.0 - (cycle - 0.85) / 0.15
            };
            let start =
                -PI / 2.0 + t * 0.15 * (k as f64 + 1.0) * if k % 2 == 0 { 1.0 } else { -1.0 };
            let gradient = ctx.create_linear_gradient(cx - r, cy - r, cx + r, cy + r);
            let _ = gradient.add_color_stop(0.0, from);
            let _ = gradient.add_color_stop(1.0, to);
            ctx.set_global_alpha(alpha.max(0.0));
            ctx.set_stroke_style_canvas_gradient(&gradient);
            ctx.set_line_cap("round");
            ctx.set_shadow_color(from);
            ctx.set_shadow_blur(22.0);
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, r, start, start + TAU * fill.max(0.001));
            ctx.stroke();
            // La tête de l'anneau : un point plus clair.
            let end = start + TAU * fill;
            ctx.set_fill_style_str("#ffffff");
            ctx.begin_path();
            let _ = ctx.arc(
                cx + r * end.cos(),
                cy + r * end.sin(),
                thickness * 0.22,
                0.0,
                TAU,
            );
            ctx.fill();
            ctx.set_shadow_blur(0.0);
            ctx.set_global_alpha(1.0);
        }

        // Le noyau : aujourd'hui.
        let pulse = 1.0 + (t * 2.4).sin() * 0.07;
        let core = base * 0.26 * pulse;
        let glow = ctx
            .create_radial_gradient(cx, cy, 0.0, cx, cy, core * 2.4)
            .ok();
        if let Some(glow) = glow {
            let _ = glow.add_color_stop(0.0, "rgba(251,191,36,0.55)");
            let _ = glow.add_color_stop(1.0, "rgba(251,191,36,0)");
            ctx.set_fill_style_canvas_gradient(&glow);
            ctx.fill_rect(cx - core * 2.4, cy - core * 2.4, core * 4.8, core * 4.8);
        }
        ctx.set_fill_style_str("#fff7e0");
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, core, 0.0, TAU);
        ctx.fill();
        ctx.set_fill_style_str(AMBER);
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, core * 0.55, 0.0, TAU);
        ctx.fill();

        // Les sept jours en orbite.
        let orbit = base * 1.32;
        for d in 0..7 {
            let a = t * 0.35 + f64::from(d) / 7.0 * TAU;
            let x = cx + orbit * a.cos();
            let y = cy + orbit * a.sin();
            let done = d < 5;
            let r = thickness
                * 0.38
                * if done {
                    1.0 + (t * 3.0 + f64::from(d)).sin().max(0.0) * 0.3
                } else {
                    0.9
                };
            ctx.begin_path();
            let _ = ctx.arc(x, y, r, 0.0, TAU);
            if done {
                ctx.set_fill_style_str(PALETTE[d as usize % PALETTE.len()]);
                ctx.set_shadow_color(PALETTE[d as usize % PALETTE.len()]);
                ctx.set_shadow_blur(14.0);
                ctx.fill();
                ctx.set_shadow_blur(0.0);
            } else {
                ctx.set_stroke_style_str("rgba(255,255,255,0.45)");
                ctx.set_line_width(2.0);
                ctx.stroke();
            }
        }
    });
    Box::new(move || drop(looping)) as Box<dyn FnOnce()>
}

// ----- La tour des jours ------------------------------------------------------------

struct Brick {
    /// Le centre horizontal, décalé de celui de la pile.
    x: f64,
    /// Le bas de la brique, au-dessus du sol (vers le haut).
    y: f64,
    vy: f64,
    rest: f64,
    landed: bool,
    /// L'écrasement à l'atterrissage, de 0 à 1.
    squash: f64,
    wobble: f64,
    color: &'static str,
    day: u32,
}

struct Spark {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    color: &'static str,
}

/// Chaque jour fait est une brique qui tombe, rebondit et s'empile ; à l'atterrissage,
/// des étincelles. `drops` augmente à chaque clic, `count` dit combien de jours compte la
/// pile.
pub fn tower(
    canvas: &paquets::active::Dom,
    drops: State<u32>,
    count: State<u32>,
) -> impl FnOnce() + use<> {
    let Some(mut surface) = Surface::new(canvas) else {
        return Box::new(|| {}) as Box<dyn FnOnce()>;
    };
    let visible = visibility(canvas);
    let calm = reduced_motion();
    let mut bricks: Vec<Brick> = Vec::new();
    let mut sparks: Vec<Spark> = Vec::new();
    let mut seen = 0u32;
    let mut auto = 0u32;
    let mut next_auto = -1.0f64;
    let mut camera = 0.0f64;
    let mut fade = 1.0f64;
    let mut clearing = false;
    let mut last = 0.0;

    let looping = animation_loop(move |time| {
        if !visible.get() || !surface.fit() {
            return;
        }
        let dt = ((time - last) / 1000.0).clamp(0.0, 0.04);
        last = time;
        let (w, h) = (surface.w, surface.h);
        let ctx = &surface.ctx;
        let bw = (w * 0.42).clamp(110.0, 190.0);
        let bh = (bw * 0.3).round();
        let ground = h - 34.0;

        // Une semaine se pose seule à la première apparition, puis chaque clic ajoute un jour.
        if next_auto < 0.0 {
            next_auto = time + 300.0;
        }
        let mut wanted = 0;
        if auto < 7 && time >= next_auto {
            auto += 1;
            wanted += 1;
            next_auto = time + if calm { 0.0 } else { 380.0 };
        }
        let clicks = drops.get();
        if clicks > seen {
            wanted += clicks - seen;
            seen = clicks;
        }
        if bricks.len() >= 18 && wanted > 0 {
            clearing = true;
        }
        if clearing {
            fade -= dt * 2.5;
            if fade <= 0.0 {
                bricks.clear();
                camera = 0.0;
                fade = 1.0;
                clearing = false;
            }
        }
        if !clearing {
            for _ in 0..wanted {
                let n = count.get();
                let below = bricks.last().map_or(0.0, |b| b.x);
                let jitter = (hash(n * 7 + 1) - 0.5) * bw * 0.16;
                let rest = bricks.len() as f64 * bh;
                bricks.push(Brick {
                    x: (below + jitter).clamp(-bw * 0.3, bw * 0.3),
                    y: rest + h + camera + 40.0,
                    vy: 0.0,
                    rest,
                    landed: calm,
                    squash: 0.0,
                    wobble: (hash(n * 13 + 5) - 0.5) * 0.3,
                    color: PALETTE[n as usize % PALETTE.len()],
                    day: n + 1,
                });
                if calm && let Some(b) = bricks.last_mut() {
                    b.y = b.rest;
                }
                count.update(|c| *c += 1);
            }
        }

        // La chute : gravité, rebond amorti, écrasement puis retour.
        for b in bricks.iter_mut() {
            if !b.landed {
                b.vy -= 2600.0 * dt;
                b.y += b.vy * dt;
                if b.y <= b.rest {
                    b.y = b.rest;
                    if b.vy < -380.0 {
                        b.vy = -b.vy * 0.28;
                        b.squash = 1.0;
                        for k in 0..10 {
                            let a = PI + f64::from(k) / 9.0 * PI;
                            let speed = 120.0 + hash(b.day * 31 + k) * 220.0;
                            sparks.push(Spark {
                                x: b.x + a.cos() * bw * 0.4,
                                y: b.rest,
                                vx: a.cos() * speed,
                                vy: -a.sin() * speed * 0.8,
                                life: 1.0,
                                color: b.color,
                            });
                        }
                    } else {
                        b.vy = 0.0;
                        b.landed = true;
                    }
                }
            }
            b.squash = (b.squash - dt * 4.0).max(0.0);
            b.wobble *= (1.0 - dt * 3.0).max(0.0);
        }
        for s in sparks.iter_mut() {
            s.vy -= 900.0 * dt;
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            s.life -= dt * 1.6;
        }
        sparks.retain(|s| s.life > 0.0);

        // La vue monte avec la pile.
        let top = bricks.len() as f64 * bh;
        let target = (top - h * 0.5).max(0.0);
        camera += (target - camera) * (dt * 4.0).min(1.0);

        // Le sol.
        let cx = w / 2.0;
        let gy = ground + camera;
        let floor = ctx.create_linear_gradient(0.0, gy, 0.0, h);
        let _ = floor.add_color_stop(0.0, "rgba(124,92,255,0.35)");
        let _ = floor.add_color_stop(1.0, "rgba(124,92,255,0)");
        ctx.set_fill_style_canvas_gradient(&floor);
        ctx.fill_rect(0.0, gy, w, h - gy.min(h));
        ctx.set_stroke_style_str("rgba(255,255,255,0.18)");
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(0.0, gy);
        ctx.line_to(w, gy);
        ctx.stroke();

        // Les briques.
        ctx.set_global_alpha(fade.max(0.0));
        ctx.set_font("700 13px Inter, system-ui, sans-serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        for b in &bricks {
            let sx = 1.0 + b.squash * 0.12;
            let sy = 1.0 - b.squash * 0.18;
            let bottom = ground + camera - b.y;
            ctx.save();
            let _ = ctx.translate(cx + b.x, bottom);
            let _ = ctx.rotate(b.wobble);
            let _ = ctx.scale(sx, sy);
            ctx.set_shadow_color(b.color);
            ctx.set_shadow_blur(18.0);
            ctx.set_fill_style_str(b.color);
            ctx.begin_path();
            ctx.round_rect_with_f64(-bw / 2.0, -bh, bw, bh - 3.0, 9.0)
                .ok();
            ctx.fill();
            ctx.set_shadow_blur(0.0);
            // Un reflet en haut de la brique.
            ctx.set_fill_style_str("rgba(255,255,255,0.22)");
            ctx.begin_path();
            ctx.round_rect_with_f64(-bw / 2.0 + 6.0, -bh + 4.0, bw - 12.0, bh * 0.28, 6.0)
                .ok();
            ctx.fill();
            ctx.set_fill_style_str("rgba(11,8,32,0.85)");
            let _ = ctx.fill_text(&format!("Jour {}", b.day), 0.0, -bh / 2.0);
            ctx.restore();
        }
        ctx.set_global_alpha(1.0);

        for s in &sparks {
            ctx.set_global_alpha(s.life.max(0.0));
            ctx.set_fill_style_str(s.color);
            ctx.begin_path();
            let _ = ctx.arc(
                cx + s.x,
                ground + camera - s.y,
                2.0 + s.life * 2.5,
                0.0,
                TAU,
            );
            ctx.fill();
        }
        ctx.set_global_alpha(1.0);
    });
    Box::new(move || drop(looping)) as Box<dyn FnOnce()>
}
