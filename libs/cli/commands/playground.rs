use crate::utils::command_error;
use crate::utils::term::get_terminal_background_color;
use chrono::{DateTime, Duration, Local, NaiveDate, Timelike};
use clap::Args;
use colored::*;
use o324_dbus::proxy::O324ServiceProxy;
use palette::{FromColor, IntoColor, Oklab, Oklch, Srgb};
use std::collections::HashMap;
use std::str::FromStr;
use terminal_size::{terminal_size, Width};

const DATE_WIDTH: usize = 17;
const HOURS_IN_DAY: usize = 24;

#[derive(Clone, Copy, Debug, Default)]
pub enum Background {
    #[default]
    Dark,
    Light,
    Custom {
        r: u8,
        g: u8,
        b: u8,
    },
}

impl FromStr for Background {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dark" => Ok(Background::Dark),
            "light" => Ok(Background::Light),
            _ => {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() == 3 {
                    let r = parts[0].trim().parse::<u8>();
                    let g = parts[1].trim().parse::<u8>();
                    let b = parts[2].trim().parse::<u8>();
                    if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
                        return Ok(Background::Custom { r, g, b });
                    }
                }
                Err(format!(
                    "Invalid theme: '{}'. Expected 'dark', 'light', or an RGB string like '15,15,35'.",
                    s
                ))
            }
        }
    }
}

impl Background {
    fn blend_color(&self, total_opacity: f32, avg_a: f32, avg_b: f32) -> Oklab {
        match self {
            Background::Dark => {
                let final_lightness = 0.05 + (total_opacity.min(1.0) * 0.55);
                Oklab::new(final_lightness, avg_a, avg_b)
            }
            Background::Light => {
                let final_lightness = 0.95 - (total_opacity.min(1.0) * 0.55);
                Oklab::new(final_lightness, avg_a, avg_b)
            }
            Background::Custom { r, g, b } => {
                let event_color = Oklab::new(0.65, avg_a, avg_b);
                let bg_srgb_f32 =
                    Srgb::new(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0);
                let bg_oklab: Oklab = Oklab::from_color(bg_srgb_f32);
                let alpha = total_opacity.min(1.0);
                let final_l = event_color.l * alpha + bg_oklab.l * (1.0 - alpha);
                let final_a = event_color.a * alpha + bg_oklab.a * (1.0 - alpha);
                let final_b = event_color.b * alpha + bg_oklab.b * (1.0 - alpha);
                Oklab::new(final_l, final_a, final_b)
            }
        }
    }
}

struct Event {
    start: DateTime<Local>,
    end: DateTime<Local>,
}

fn generate_events() -> Vec<Event> {
    let now = Local::now();
    let yesterday = now - Duration::days(1);
    vec![
        Event {
            start: now.with_hour(14).unwrap().with_minute(5).unwrap(),
            end: now.with_hour(16).unwrap().with_minute(55).unwrap(),
        },
        Event {
            start: yesterday.with_hour(10).unwrap().with_minute(0).unwrap(),
            end: yesterday.with_hour(11).unwrap().with_minute(9).unwrap(),
        },
        Event {
            start: yesterday.with_hour(11).unwrap().with_minute(9).unwrap(),
            end: yesterday.with_hour(12).unwrap().with_minute(15).unwrap(),
        },
        Event {
            start: now - Duration::days(2),
            end: now - Duration::days(2) + Duration::minutes(70),
        },
    ]
}

#[derive(Clone, Copy, Debug)]
struct EventInSlot {
    hue: f32,
    opacity: f32,
}

struct Day {
    date_str: String,
    base_states: Vec<u8>,
    slots: Vec<Vec<EventInSlot>>,
    chart_width: usize,
    slot_width: usize,
}

type EventKey = (i64, i64);
type HueMap = HashMap<EventKey, f32>;

impl Day {
    pub fn new(date_str: String, chart_width: usize, slot_width: usize) -> Self {
        let slots = vec![Vec::new(); chart_width];
        let mut base_states = vec![0; chart_width];
        for hour in 0..HOURS_IN_DAY {
            let is_working_hour = (9..17).contains(&hour);
            let start_char = hour * slot_width;
            let end_char = ((hour + 1) * slot_width).min(chart_width);
            for state in base_states.iter_mut().take(end_char).skip(start_char) {
                *state = if is_working_hour { 1 } else { 0 };
            }
        }
        Self {
            date_str,
            base_states,
            slots,
            chart_width,
            slot_width,
        }
    }

    pub fn insert_event(&mut self, event: &Event, current_day: NaiveDate, hue_map: &HueMap) {
        let day_start = current_day
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();
        let day_end = day_start + Duration::days(1);

        if event.end <= day_start || event.start >= day_end {
            return;
        }

        let effective_start = event.start.max(day_start);
        let effective_end = event.end.min(day_end);
        let start_minute_of_day = effective_start.hour() * 60 + effective_start.minute();
        let end_minute_of_day = effective_end.hour() * 60 + effective_end.minute();
        let total_minutes_in_day = (HOURS_IN_DAY * 60) as u32;
        let total_chars_in_day = self.chart_width as u32;
        let start_char_idx = (start_minute_of_day * total_chars_in_day) / total_minutes_in_day;
        let end_char_idx =
            ((end_minute_of_day.saturating_sub(1)) * total_chars_in_day) / total_minutes_in_day;
        let event_hue = get_event_hue(event, hue_map);

        for char_idx in start_char_idx..=end_char_idx {
            if char_idx as usize >= self.chart_width {
                continue;
            }
            let char_start_minute = (char_idx * total_minutes_in_day) / total_chars_in_day;
            let char_end_minute = ((char_idx + 1) * total_minutes_in_day) / total_chars_in_day;
            if char_end_minute == char_start_minute {
                continue;
            }
            let minutes_in_this_char = char_end_minute - char_start_minute;
            let overlap_start = start_minute_of_day.max(char_start_minute);
            let overlap_end = end_minute_of_day.min(char_end_minute);
            let overlap_minutes = overlap_end.saturating_sub(overlap_start);
            if overlap_minutes > 0 {
                let opacity = overlap_minutes as f32 / minutes_in_this_char as f32;
                let event_in_slot = EventInSlot {
                    hue: event_hue,
                    opacity,
                };
                if let Some(slot_list) = self.slots.get_mut(char_idx as usize) {
                    slot_list.push(event_in_slot);
                }
            }
        }
    }
}

fn get_event_hue(event: &Event, hue_map: &HueMap) -> f32 {
    let event_key = (event.start.timestamp(), event.end.timestamp());
    *hue_map.get(&event_key).unwrap_or(&0.0)
}

fn blend_and_get_color(events_in_slot: &[EventInSlot], theme: Background) -> (u8, u8, u8) {
    let mut total_opacity = 0.0;
    let mut weighted_a = 0.0;
    let mut weighted_b = 0.0;
    for event in events_in_slot {
        total_opacity += event.opacity;
        let oklab_color: Oklab = Oklch::new(0.2, 0.16, event.hue).into_color();
        weighted_a += oklab_color.a * event.opacity;
        weighted_b += oklab_color.b * event.opacity;
    }
    if total_opacity == 0.0 {
        return (0, 0, 0);
    }
    let avg_a = weighted_a / total_opacity;
    let avg_b = weighted_b / total_opacity;
    let final_color = theme.blend_color(total_opacity, avg_a, avg_b);
    let srgb_u8: Srgb<u8> = Srgb::from_color(final_color).into_format();
    (srgb_u8.red, srgb_u8.green, srgb_u8.blue)
}

fn print_day(day: &Day, theme: Background) {
    let mut line = String::with_capacity(DATE_WIDTH + day.chart_width);
    let colored_date_part = day.date_str.normal().to_string();
    line.push_str(&colored_date_part);
    let visible_date_len = day.date_str.chars().count();
    if visible_date_len < DATE_WIDTH {
        for _ in 0..(DATE_WIDTH - visible_date_len) {
            line.push(' ');
        }
    }
    let base_char_list: Vec<char> = (0..day.chart_width)
        .map(|i| if i % day.slot_width == 0 { ':' } else { '·' })
        .collect();
    for (i, base_char) in base_char_list.iter().enumerate().take(day.chart_width) {
        let slot_events = &day.slots[i];
        let base_state = day.base_states[i];
        let char_to_print = base_char;
        if !slot_events.is_empty() {
            let (r, g, b) = blend_and_get_color(slot_events, theme);
            line.push_str(
                &char_to_print
                    .to_string()
                    .white()
                    .on_truecolor(r, g, b)
                    .to_string(),
            );
        } else {
            let colored_char = if base_state == 0 {
                char_to_print.to_string().bright_black()
            } else {
                char_to_print.to_string().normal()
            };
            line.push_str(&colored_char.to_string());
        }
    }
    println!("{}", line);
}

#[derive(Args, Debug)]
pub struct Command {
    #[arg(long)]
    pub theme: Option<Background>,
}

pub async fn handle(command: Command, _proxy: O324ServiceProxy<'_>) -> command_error::Result<()> {
    let theme = command.theme.unwrap_or_else(|| {
        get_terminal_background_color()
            .and_then(|option_color| option_color.ok_or_else(|| "No color detected".into()))
            .map(|e| Background::Custom {
                r: e.r,
                g: e.g,
                b: e.b,
            })
            .unwrap_or(Background::Dark)
    });

    let terminal_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };
    let available_width = terminal_width.saturating_sub(DATE_WIDTH);
    if available_width < HOURS_IN_DAY {
        println!("terminal is too narrow to display the calendar.");
        return Ok(());
    }

    let slot_width = available_width / HOURS_IN_DAY;
    let chart_width = slot_width * HOURS_IN_DAY;

    let events = generate_events();
    //let mut hue_generator = SpacedRandomGenerator::new(359, 3.0);
    let mut event_hues: HueMap = HashMap::new();

    for event in &events {
        let event_key = (event.start.timestamp(), event.end.timestamp());
        if let std::collections::hash_map::Entry::Vacant(e) = event_hues.entry(event_key) {
            // TODO
            e.insert(200 as f32);
            //if let Some(hue) = hue_generator.next_number() {
            //    e.insert(hue as f32);
            //}
        }
    }

    let mut tens_line_chars = vec![' '; chart_width];
    let mut units_line_chars = vec![' '; chart_width];
    for hour in 0..HOURS_IN_DAY {
        let target_pos = hour * slot_width;
        if let Some(c) = std::char::from_digit((hour % 10) as u32, 10) {
            units_line_chars[target_pos] = c;
        }
        if hour >= 10 {
            if let Some(c) = std::char::from_digit((hour / 10) as u32, 10) {
                tens_line_chars[target_pos] = c;
            }
        }
    }
    let tens_line_str = tens_line_chars.iter().collect::<String>();
    let units_line_str = units_line_chars.iter().collect::<String>();
    let padded_tens = format!("{:<width$}{}", "", tens_line_str, width = DATE_WIDTH);
    let padded_units = format!("{:<width$}{}", "", units_line_str, width = DATE_WIDTH);
    println!("{}", padded_tens.normal());
    println!("{}", padded_units.normal());

    let today = Local::now().date_naive();
    for i in 0..7 {
        let day_offset = 6 - i;
        let current_day_date = today - Duration::days(day_offset);
        let date_key = current_day_date.format("%y/%m/%d (%a)").to_string();
        let mut day = Day::new(date_key, chart_width, slot_width);
        for event in &events {
            day.insert_event(event, current_day_date, &event_hues);
        }
        print_day(&day, theme);
    }
    Ok(())
}
