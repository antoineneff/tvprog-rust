use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveTime};
use reqwest::blocking::get;
use serde::Deserialize;

#[derive(Debug)]
struct Program {
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
    title: String,
    channel: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct XMLChannel {
    id: String,
    #[serde(rename = "display-name")]
    display_name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct XMLProgram {
    start: String,
    stop: String,
    channel: String,
    title: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Xml {
    #[serde(rename = "$unflatten=channel")]
    channels: Vec<XMLChannel>,
    #[serde(rename = "$unflatten=programme")]
    programs: Vec<XMLProgram>,
}

const CHANNELS: [&str; 19] = [
    "TF1",
    "France 2",
    "France 3",
    "Canal+",
    "France 5",
    "M6",
    "Arte",
    "C8",
    "W9",
    "TMC",
    "TFX",
    "NRJ 12",
    "France 4",
    "CSTAR",
    "L'Equipe",
    "6ter",
    "RMC Story",
    "RMC Découverte",
    "Chérie 25",
];

fn filter_programs(xml: &Xml) -> Vec<Program> {
    let filtered_channel_ids: Vec<String> = filter_channel_ids(&xml.channels);

    xml.programs
        .iter()
        .filter(|program| filtered_channel_ids.contains(&program.channel))
        .filter(|program| is_evening_program(&program.start, &program.stop))
        .map(|program| Program {
            start: DateTime::parse_from_str(&program.start, "%Y%m%d%H%M%S %z").unwrap(),
            end: DateTime::parse_from_str(&program.stop, "%Y%m%d%H%M%S %z").unwrap(),
            title: program.title.to_owned(),
            channel: channel_id_to_name(&program.channel, &xml.channels).to_string(),
        })
        .collect()
}

fn filter_channel_ids(channels: &Vec<XMLChannel>) -> Vec<String> {
    channels
        .into_iter()
        .filter(|channel| CHANNELS.contains(&channel.display_name.as_str()))
        .map(|channel| channel.id.to_owned())
        .collect()
}

fn is_evening_program(start_date: &str, end_date: &str) -> bool {
    let minimum_program_start: NaiveTime = NaiveTime::from_hms(20, 45, 0);
    let maximum_program_start: NaiveTime = NaiveTime::from_hms(21, 20, 0);
    let now = Local::today();
    let start_parsed = DateTime::parse_from_str(start_date, "%Y%m%d%H%M%S %z").unwrap();
    let end_parsed = DateTime::parse_from_str(end_date, "%Y%m%d%H%M%S %z").unwrap();
    let duration = end_parsed.signed_duration_since(start_parsed);

    now.year() == start_parsed.year()
        && now.month() == start_parsed.month()
        && now.day() == start_parsed.day()
        && start_parsed.time() > minimum_program_start
        && start_parsed.time() < maximum_program_start
        && duration.num_minutes() > 35
}

fn channel_id_to_name<'a>(channel_id: &str, channels: &'a Vec<XMLChannel>) -> &'a str {
    let found_channel = channels
        .into_iter()
        .find(|channel| channel.id == channel_id)
        .unwrap();
    &found_channel.display_name
}

fn pretty_print(programs: &Vec<Program>) {
    println!("┌{}┬{}┬{}┐", "─".repeat(16), "─".repeat(57), "─".repeat(15));
    println!("│ {:14} │ {:55} │ {:13} │", "Chaine", "Titre", "Horaires");
    println!("├{}┼{}┼{}┤", "─".repeat(16), "─".repeat(57), "─".repeat(15));
    for program in programs {
        println!(
            "│ {:14} │ {:55} │ {} - {} │",
            program.channel,
            str_truncate(&program.title, 55),
            program.start.format("%H:%M"),
            program.end.format("%H:%M")
        )
    }
    println!("└{}┴{}┴{}┘", "─".repeat(16), "─".repeat(57), "─".repeat(15));
}

fn str_truncate(string: &str, limit: u32) -> String {
    string.chars().take(limit as usize).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = get("https://xmltv.ch/xmltv/xmltv-tnt.xml")?;
    let xml: Xml = quick_xml::de::from_slice(&res.bytes()?)?;
    let filtered_programs: Vec<Program> = filter_programs(&xml);

    pretty_print(&filtered_programs);

    Ok(())
}
