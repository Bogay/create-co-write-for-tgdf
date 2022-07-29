use futures::future::try_join_all;
use iter_tools::Itertools;
use once_cell::sync::Lazy;
use reqwest::IntoUrl;
use scraper::{Html, Selector};
use serde::Serialize;
use std::{fmt::Display, str::FromStr};

// ref: https://github.com/causal-agent/scraper/issues/53
macro_rules! selector {
    ($e: expr) => {{
        static SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse($e).unwrap());
        &*SELECTOR
    }};
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Clone, Copy)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

#[derive(Debug)]
struct TimeParseError {}

impl Display for TimeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TimeParseError")
    }
}

impl std::error::Error for TimeParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl FromStr for Time {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let h_and_m = s
            .split(":")
            .map(|v| v.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        let (hour, minute) = h_and_m
            .into_iter()
            .collect_tuple::<(u8, u8)>()
            .ok_or(TimeParseError {})?;
        Ok(Time { hour, minute })
    }
}

#[derive(Debug, Serialize)]
pub struct Presenter {
    pub name: String,
    pub title: String,
    pub link: String,
    // pub avatar_link: String,
    pub introduction: String,
}

impl Presenter {
    pub async fn from_url<T: IntoUrl + Clone>(url: T) -> Result<Self, Box<dyn std::error::Error>> {
        let content = reqwest::get(url.clone()).await?.text().await?;
        let presneter = Html::parse_document(&content);

        let name = presneter
            .select(selector!(".speaker__name"))
            .next()
            .unwrap()
            .text()
            .collect::<String>();
        let title = presneter
            .select(selector!(".speaker__title"))
            .next()
            .unwrap()
            .text()
            .collect::<String>();
        let introduction = presneter
            .select(selector!(".speaker__introduce"))
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        Ok(Self {
            name,
            title,
            link: url.into_url().unwrap().to_string(),
            introduction,
        })
    }
}

#[derive(Serialize, Debug)]
pub struct Session {
    pub day: u32,
    pub track: u32,
    pub time: (Time, Time),
    pub title: String,
    pub presenters: Vec<Presenter>,
    pub tags: Vec<String>,
    pub description: String,
}

pub async fn fetch() -> Result<Vec<Session>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let agenda = client
        .get("https://2021.tgdf.tw/agenda")
        .send()
        .await?
        .text()
        .await?;
    let agenda = Html::parse_document(&agenda);
    let agenda_selector = selector!(".agenda__row");
    let mut collected_sessions = vec![];
    for session_row in agenda.select(agenda_selector) {
        let session_time = session_row
            .select(selector!(".agenda__time"))
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .split(" - ")
            .map(|v| v.parse::<Time>())
            .collect::<Result<Vec<_>, _>>()?;
        let (time_from, time_to) = session_time
            .into_iter()
            .collect_tuple()
            .ok_or(TimeParseError {})?;
        let session_selector = selector!(".agenda__sessions");
        for (i, sessions) in session_row.select(session_selector).enumerate() {
            let track = (i + 1) as u32;
            let session_selector = &selector!(".session");
            for session in sessions.select(session_selector) {
                let tags = session
                    .select(selector!(".session__badge"))
                    .map(|v| v.text().collect::<String>())
                    .collect::<Vec<_>>();
                let presenter_links = session.select(selector!("p")).next().map(|v| {
                    v.select(selector!("a"))
                        .map(|link| {
                            format!("https://2021.tgdf.tw{}", link.value().attr("href").unwrap())
                        })
                        .collect::<Vec<_>>()
                });
                let presenter_links = match presenter_links {
                    Some(links) => links,
                    None => continue,
                };
                if presenter_links.is_empty() {
                    continue;
                }
                let presenters =
                    try_join_all(presenter_links.iter().map(Presenter::from_url)).await?;
                let title = session
                    .select(selector!("h6"))
                    .next()
                    .unwrap()
                    .text()
                    .collect::<String>();
                let session_url = session
                    .select(selector!("a"))
                    .next()
                    .unwrap()
                    .value()
                    .attr("href")
                    .unwrap();
                let session_url = format!("https://2021.tgdf.tw{}", session_url);
                collected_sessions.push(Session {
                    title,
                    track,
                    tags,
                    presenters,
                    time: (time_from, time_to),
                    day: 1,
                    description: fetch_session_description(session_url).await?,
                });
            }
        }
    }

    Ok(collected_sessions)
}

async fn fetch_session_description<T: IntoUrl>(
    url: T,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = reqwest::get(url).await?.text().await?;
    let content = Html::parse_document(&content);
    Ok(content
        .select(selector!(".session__description"))
        .next()
        .map(|e| e.html())
        .unwrap_or_default())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_time_order() {
        assert!(
            Time {
                hour: 10,
                minute: 10
            } < Time {
                hour: 11,
                minute: 0
            }
        );
    }
}
