//! Le cœur de Routia, sans navigateur ni serveur : les dates, les habitudes, les séries
//! (*streaks*), l'historique et l'export. Le même code tourne dans les tests, sur le
//! serveur (rendu de la page) et dans le navigateur (WebAssembly).

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

// ----- Dates ------------------------------------------------------------------------

/// Un jour du calendrier, compté depuis le 1er janvier 1970.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Day(pub i32);

impl Day {
    /// Le jour d'une date du calendrier grégorien.
    pub fn from_ymd(y: i32, m: u32, d: u32) -> Day {
        // Howard Hinnant, « days_from_civil ».
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = m as i32;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d as i32 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        Day(era * 146_097 + doe - 719_468)
    }

    /// L'année, le mois (1 à 12) et le jour (1 à 31).
    pub fn ymd(self) -> (i32, u32, u32) {
        let z = self.0 + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let y = yoe + era * 400 + i32::from(m <= 2);
        (y, m, d)
    }

    /// `AAAA-MM-JJ`.
    pub fn parse(s: &str) -> Option<Day> {
        let mut parts = s.trim().splitn(3, '-');
        let y = parts.next()?.parse().ok()?;
        let m: u32 = parts.next()?.parse().ok()?;
        let d: u32 = parts.next()?.parse().ok()?;
        if !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
            return None;
        }
        Some(Day::from_ymd(y, m, d))
    }

    /// Le jour à partir d'un horodatage en millisecondes (`Date.now()`), dans un fuseau
    /// décalé de `offset_minutes` par rapport à UTC (positif à l'est).
    pub fn from_millis(ms: f64, offset_minutes: i32) -> Day {
        let local = ms + f64::from(offset_minutes) * 60_000.0;
        Day((local / 86_400_000.0).floor() as i32)
    }

    pub fn shift(self, days: i32) -> Day {
        Day(self.0 + days)
    }

    /// 0 pour lundi … 6 pour dimanche.
    pub fn weekday(self) -> u32 {
        // Le 1er janvier 1970 était un jeudi.
        (self.0 + 3).rem_euclid(7) as u32
    }

    /// Le lundi de la semaine.
    pub fn week_start(self) -> Day {
        self.shift(-(self.weekday() as i32))
    }

    pub fn key(self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (y, m, d) = self.ymd();
        write!(f, "{y:04}-{m:02}-{d:02}")
    }
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

pub const WEEKDAYS: [&str; 7] = ["L", "M", "M", "J", "V", "S", "D"];
pub const MONTHS: [&str; 12] = [
    "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
    "déc.",
];

// ----- Habitudes --------------------------------------------------------------------

/// Les couleurs d'une habitude, choisies à la création.
pub const COLORS: [&str; 6] = [
    "#7c5cff", "#22d3ee", "#34d399", "#fbbf24", "#f472b6", "#fb7185",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Habit {
    pub id: String,
    pub name: String,
    /// Le nombre de fois par semaine : 7, c'est chaque jour.
    pub frequency: u8,
    /// L'indice dans [`COLORS`].
    #[serde(default)]
    pub color: u8,
    /// `AAAA-MM-JJ` : le jour de création (les jours d'avant ne sont pas « manqués »).
    pub created: String,
    /// `"2026-11-10": true` : faite ce jour-là.
    #[serde(default)]
    pub history: BTreeMap<String, bool>,
}

/// L'unité d'une série : des jours pour une habitude quotidienne, des semaines sinon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Days,
    Weeks,
}

impl Unit {
    pub fn label(self, n: u32) -> &'static str {
        match (self, n) {
            (Unit::Days, 0 | 1) => "jour",
            (Unit::Days, _) => "jours",
            (Unit::Weeks, 0 | 1) => "semaine",
            (Unit::Weeks, _) => "semaines",
        }
    }
}

/// L'état d'une case de l'historique.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Done,
    Missed,
    /// Aujourd'hui, pas encore fait : encore possible.
    Pending,
    /// Pas compté : un jour de repos d'une habitude hebdomadaire.
    Rest,
    /// Avant la création, ou dans le futur.
    Empty,
}

impl Cell {
    pub fn class(self) -> &'static str {
        match self {
            Cell::Done => "is-done",
            Cell::Missed => "is-missed",
            Cell::Pending => "is-pending",
            Cell::Rest => "is-rest",
            Cell::Empty => "is-empty",
        }
    }
}

impl Habit {
    pub fn new(id: impl Into<String>, name: &str, frequency: u8, color: u8, today: Day) -> Habit {
        Habit {
            id: id.into(),
            name: clean_name(name),
            frequency: frequency.clamp(1, 7),
            color: color % COLORS.len() as u8,
            created: today.key(),
            history: BTreeMap::new(),
        }
    }

    pub fn color(&self) -> &'static str {
        COLORS[usize::from(self.color) % COLORS.len()]
    }

    pub fn created_day(&self) -> Option<Day> {
        Day::parse(&self.created)
    }

    pub fn is_done(&self, day: Day) -> bool {
        self.history.get(&day.key()).copied().unwrap_or(false)
    }

    pub fn set_done(&mut self, day: Day, done: bool) {
        if done {
            self.history.insert(day.key(), true);
            // Cocher un jour avant la création la recule : l'historique reste cohérent.
            if self.created_day().is_none_or(|c| day < c) {
                self.created = day.key();
            }
        } else {
            self.history.remove(&day.key());
        }
    }

    pub fn toggle(&mut self, day: Day) {
        let done = self.is_done(day);
        self.set_done(day, !done);
    }

    pub fn daily(&self) -> bool {
        self.frequency >= 7
    }

    pub fn unit(&self) -> Unit {
        if self.daily() {
            Unit::Days
        } else {
            Unit::Weeks
        }
    }

    /// Le nombre de jours faits dans la semaine qui commence le lundi `monday`.
    pub fn done_in_week(&self, monday: Day) -> u32 {
        (0..7).filter(|i| self.is_done(monday.shift(*i))).count() as u32
    }

    /// La série en cours, à `today`. Un jour (ou une semaine) pas encore fini ne casse
    /// pas la série : il la prolonge seulement une fois fait.
    pub fn streak(&self, today: Day) -> u32 {
        let start = self.created_day().unwrap_or(today);
        if self.daily() {
            let mut day = if self.is_done(today) {
                today
            } else {
                today.shift(-1)
            };
            let mut n = 0;
            while day >= start.shift(-366 * 10) && self.is_done(day) {
                n += 1;
                day = day.shift(-1);
            }
            n
        } else {
            let target = u32::from(self.frequency);
            let this_week = today.week_start();
            let mut week = if self.done_in_week(this_week) >= target {
                this_week
            } else {
                this_week.shift(-7)
            };
            let mut n = 0;
            while week >= start.week_start() && self.done_in_week(week) >= target {
                n += 1;
                week = week.shift(-7);
            }
            n
        }
    }

    /// La plus longue série, de la création à `today`.
    pub fn best_streak(&self, today: Day) -> u32 {
        let start = self.created_day().unwrap_or(today);
        let mut best = 0;
        let mut run = 0;
        if self.daily() {
            let mut day = start;
            while day <= today {
                if self.is_done(day) {
                    run += 1;
                    best = best.max(run);
                } else if day != today {
                    run = 0;
                }
                day = day.shift(1);
            }
        } else {
            let target = u32::from(self.frequency);
            let mut week = start.week_start();
            while week <= today {
                if self.done_in_week(week) >= target {
                    run += 1;
                    best = best.max(run);
                } else if week != today.week_start() {
                    run = 0;
                }
                week = week.shift(7);
            }
        }
        best
    }

    /// L'état de `day` dans l'historique.
    pub fn cell(&self, day: Day, today: Day) -> Cell {
        let created = self.created_day().unwrap_or(today);
        if self.is_done(day) {
            return Cell::Done;
        }
        if day > today || day < created {
            return Cell::Empty;
        }
        if day == today {
            return Cell::Pending;
        }
        if self.daily() {
            return Cell::Missed;
        }
        // Une semaine passée qui n'a pas atteint l'objectif : ses jours vides sont
        // manqués. Une semaine réussie, ou la semaine en cours : des jours de repos.
        let monday = day.week_start();
        if monday < today.week_start() && self.done_in_week(monday) < u32::from(self.frequency) {
            Cell::Missed
        } else {
            Cell::Rest
        }
    }

    /// Les `days` derniers jours, du plus ancien à `today`.
    pub fn history_cells(&self, today: Day, days: i32) -> Vec<(Day, Cell)> {
        (0..days)
            .rev()
            .map(|i| {
                let day = today.shift(-i);
                (day, self.cell(day, today))
            })
            .collect()
    }

    /// Les jours manqués sur les `days` derniers jours.
    pub fn missed(&self, today: Day, days: i32) -> u32 {
        self.history_cells(today, days)
            .iter()
            .filter(|(_, c)| *c == Cell::Missed)
            .count() as u32
    }

    /// La part des jours comptés (faits ou manqués) qui ont été faits, en pourcents.
    pub fn rate(&self, today: Day, days: i32) -> u32 {
        let cells = self.history_cells(today, days);
        let done = cells.iter().filter(|(_, c)| *c == Cell::Done).count();
        let missed = cells.iter().filter(|(_, c)| *c == Cell::Missed).count();
        (done * 100).checked_div(done + missed).unwrap_or(0) as u32
    }

    pub fn frequency_label(&self) -> String {
        frequency_label(self.frequency)
    }
}

pub fn frequency_label(frequency: u8) -> String {
    match frequency {
        7.. => "Chaque jour".into(),
        1 => "1× par semaine".into(),
        n => format!("{n}× par semaine"),
    }
}

/// Un nom propre : sans espaces en trop, 40 caractères au plus.
pub fn clean_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(40)
        .collect()
}

// ----- Stockage et export -----------------------------------------------------------

/// La clé `localStorage` des habitudes.
pub const STORAGE_KEY: &str = "routia.habits.v1";

/// Lit les habitudes enregistrées ; une valeur illisible donne une liste vide.
pub fn load(json: &str) -> Vec<Habit> {
    serde_json::from_str::<Vec<Habit>>(json)
        .map(|list| {
            list.into_iter()
                .filter(|h| !h.name.is_empty())
                .map(|mut h| {
                    h.frequency = h.frequency.clamp(1, 7);
                    h.history.retain(|k, v| *v && Day::parse(k).is_some());
                    h
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn save(habits: &[Habit]) -> String {
    serde_json::to_string(habits).unwrap_or_else(|_| "[]".into())
}

/// La meilleure série en cours, toutes habitudes confondues.
pub fn top_streak(habits: &[Habit], today: Day) -> Option<(&Habit, u32)> {
    habits
        .iter()
        .map(|h| (h, h.streak(today)))
        .max_by_key(|(_, s)| *s)
}

/// Le bilan en texte : une ligne par habitude et les 4 dernières semaines en cases.
pub fn export_text(habits: &[Habit], today: Day) -> String {
    let (y, m, d) = today.ymd();
    let mut out = format!("Routia — bilan du {d} {} {y}\n\n", MONTHS[(m - 1) as usize]);
    if habits.is_empty() {
        out.push_str("Aucune habitude pour l'instant.\n");
    }
    for h in habits {
        let streak = h.streak(today);
        let grid: String = h
            .history_cells(today, 28)
            .iter()
            .map(|(_, c)| match c {
                Cell::Done => '■',
                Cell::Missed => '×',
                Cell::Pending => '◇',
                Cell::Rest => '·',
                Cell::Empty => ' ',
            })
            .collect();
        out.push_str(&format!(
            "• {} ({})\n  Série : {streak} {} · record : {} · réussite (30 j) : {} % · manqués : {}\n  {}\n\n",
            h.name,
            h.frequency_label().to_lowercase(),
            h.unit().label(streak),
            h.best_streak(today),
            h.rate(today, 30),
            h.missed(today, 30),
            grid.trim_end(),
        ));
    }
    out.push_str("■ fait · × manqué · ◇ aujourd'hui · · repos\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> Day {
        Day::parse(s).unwrap()
    }

    #[test]
    fn dates_round_trip() {
        assert_eq!(Day::from_ymd(1970, 1, 1), Day(0));
        assert_eq!(day("2026-11-10").to_string(), "2026-11-10");
        assert_eq!(day("2024-02-29").shift(1).to_string(), "2024-03-01");
        assert_eq!(day("1999-12-31").shift(1).to_string(), "2000-01-01");
        assert!(Day::parse("2025-02-29").is_none());
        assert!(Day::parse("2025-13-01").is_none());
        assert!(Day::parse("hier").is_none());
        for n in -800_000..800_000 {
            if n % 997 == 0 {
                let (y, m, d) = Day(n).ymd();
                assert_eq!(Day::from_ymd(y, m, d), Day(n));
            }
        }
    }

    #[test]
    fn weeks_start_on_monday() {
        // Le 10 novembre 2026 est un mardi.
        assert_eq!(day("2026-11-10").weekday(), 1);
        assert_eq!(day("2026-11-10").week_start(), day("2026-11-09"));
        assert_eq!(day("2026-11-15").week_start(), day("2026-11-09"));
        assert_eq!(day("2026-11-16").week_start(), day("2026-11-16"));
    }

    #[test]
    fn time_zones_move_midnight() {
        // 23 h 30 UTC le 10 : déjà le 11 à Paris (UTC+1).
        let ms = f64::from(day("2026-11-10").0) * 86_400_000.0 + 23.5 * 3_600_000.0;
        assert_eq!(Day::from_millis(ms, 0), day("2026-11-10"));
        assert_eq!(Day::from_millis(ms, 60), day("2026-11-11"));
    }

    #[test]
    fn daily_streaks() {
        let today = day("2026-11-10");
        let mut h = Habit::new("1", "  Méditer   10 min ", 7, 0, today.shift(-10));
        assert_eq!(h.name, "Méditer 10 min");
        assert_eq!(h.streak(today), 0);
        for i in 1..=3 {
            h.set_done(today.shift(-i), true);
        }
        // Aujourd'hui pas encore coché : la série d'hier tient toujours.
        assert_eq!(h.streak(today), 3);
        assert_eq!(h.cell(today, today), Cell::Pending);
        h.toggle(today);
        assert_eq!(h.streak(today), 4);
        // Un trou casse la série.
        h.set_done(today.shift(-2), false);
        assert_eq!(h.streak(today), 2);
        assert_eq!(h.cell(today.shift(-2), today), Cell::Missed);
        assert_eq!(h.best_streak(today), 2);
        h.set_done(today.shift(-2), true);
        assert_eq!(h.best_streak(today), 4);
        // Avant la création : rien de manqué.
        assert_eq!(h.cell(today.shift(-30), today), Cell::Empty);
        assert_eq!(h.missed(today, 30), 7);
    }

    #[test]
    fn weekly_streaks() {
        let today = day("2026-11-12"); // jeudi
        let mut h = Habit::new("2", "Courir", 3, 1, day("2026-10-19"));
        // Trois semaines complètes, trois fois chacune.
        for monday in ["2026-10-19", "2026-10-26", "2026-11-02"] {
            for i in [0, 2, 4] {
                h.set_done(day(monday).shift(i), true);
            }
        }
        // La semaine en cours n'est pas finie : elle ne casse rien.
        assert_eq!(h.streak(today), 3);
        assert_eq!(h.unit().label(3), "semaines");
        assert_eq!(h.cell(day("2026-10-20"), today), Cell::Rest);
        h.set_done(day("2026-11-09"), true);
        h.set_done(day("2026-11-10"), true);
        h.set_done(day("2026-11-11"), true);
        assert_eq!(h.streak(today), 4);
        assert_eq!(h.best_streak(today), 4);
        // Une semaine ratée : ses jours vides sont manqués.
        h.set_done(day("2026-10-28"), false);
        assert_eq!(h.streak(today), 2);
        assert_eq!(h.cell(day("2026-10-27"), today), Cell::Missed);
    }

    #[test]
    fn checking_an_old_day_moves_the_creation() {
        let today = day("2026-11-10");
        let mut h = Habit::new("3", "Lire", 7, 2, today);
        h.set_done(today.shift(-1), true);
        assert_eq!(h.created, "2026-11-09");
        assert_eq!(h.streak(today), 1);
    }

    #[test]
    fn storage_survives_bad_data() {
        let today = day("2026-11-10");
        let mut h = Habit::new("4", "Boire de l'eau", 9, 7, today);
        assert_eq!(h.frequency, 7);
        assert!(usize::from(h.color) < COLORS.len());
        h.set_done(today, true);
        let json = save(std::slice::from_ref(&h));
        assert_eq!(load(&json), vec![h]);
        assert!(load("pas du json").is_empty());
        let odd = r#"[{"id":"x","name":"A","frequency":0,"created":"2026-11-01","history":{"n'importe":true,"2026-11-02":false}}]"#;
        let list = load(odd);
        assert_eq!(list[0].frequency, 1);
        assert!(list[0].history.is_empty());
    }

    #[test]
    fn exports_a_readable_summary() {
        let today = day("2026-11-10");
        let mut h = Habit::new("5", "Méditer", 7, 0, today.shift(-5));
        h.set_done(today, true);
        h.set_done(today.shift(-1), true);
        let text = export_text(&[h], today);
        assert!(text.starts_with("Routia — bilan du 10 nov. 2026"));
        assert!(text.contains("• Méditer (chaque jour)"));
        assert!(text.contains("Série : 2 jours"));
        assert!(text.contains("■■"));
        assert!(export_text(&[], today).contains("Aucune habitude"));
    }
}
