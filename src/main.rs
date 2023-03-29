// use std::error::Error;
use chrono::Datelike;
// Use reqwest headers to get X-Scope
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize)]
struct Response {
    data: Data,
}
#[derive(Debug, Deserialize)]
struct Data {
    signature: String,
}
#[derive(Debug, Deserialize)]
struct ResponseKey {
    data: Key,
}
#[derive(Debug, Deserialize)]
struct Key {
    key: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct Post {
    renderKey: String,
    host: String,
    unitGuid: String,
    scheduleDay: u32,
    width: u64,
    height: u64,
    selectionType: u32,
    selection: String,
    week: u32,
    year: u32,
}
#[derive(Debug, Deserialize)]
struct ScheduleData {
    data: Schedule,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Schedule {
    lessonInfo: Vec<Lesson>,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Lesson {
    timeStart: String,
    timeEnd: String,
    texts: Vec<String>,
}

fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <class id>", args[0]);
        std::process::exit(1);
    }
    let current_time = chrono::offset::Local::now();
    let mut weekday = current_time.date_naive().weekday().number_from_monday();
    if !(1..=5).contains(&weekday) {
        weekday = 0;
    }
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "X-Scope",
        "8a22163c-8662-4535-9050-bc5e1923df48".parse().unwrap(),
    );
    headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    // Class ID or personal ID
    let signaturestring = format!("{{'signature':'{}'}}", args[1]);
    let res: Response = client
        .post("https://web.skola24.se/api/encrypt/signature")
        .headers(headers.clone())
        .body(signaturestring)
        .send()?
        .json()?;

    let reskey: ResponseKey = client
        .post("https://web.skola24.se/api/get/timetable/render/key")
        .headers(headers.clone())
        .body("null")
        .send()?
        .json()?;

    let new_post = Post {
        renderKey: reskey.data.key,
        host: "it-gymnasiet.skola24.se".to_string(),
        unitGuid: "ZTEyNTdlZjItZDc3OC1mZWJkLThiYmEtOGYyZDA4NGU1YjI2".to_string(),
        scheduleDay: weekday,
        width: 1,
        height: 1,
        selectionType: 4,
        selection: res.data.signature,
        week: current_time.iso_week().week(),
        year: 2023,
    };

    let mut new_post: ScheduleData = client
        .post("https://web.skola24.se/api/render/timetable")
        .headers(headers)
        .json(&new_post)
        .send()?
        .json()?;

    new_post
        .data
        .lessonInfo
        .sort_by(|a, b| a.timeStart.cmp(&b.timeStart));
    for lesson in new_post.data.lessonInfo {
        println!(
            "{}, börjar kl {} och slutar kl {} i sal {}",
            lesson.texts[0], lesson.timeStart, lesson.timeEnd, lesson.texts[2]
        );
    }

    Ok(())
}
