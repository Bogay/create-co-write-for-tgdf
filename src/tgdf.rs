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

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Clone, Copy)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}:{:02}", self.hour, self.minute))
    }
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
            .split(':')
            .map(|v| v.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        let (hour, minute) = h_and_m
            .into_iter()
            .collect_tuple::<(u8, u8)>()
            .ok_or(TimeParseError {})?;
        Ok(Time { hour, minute })
    }
}

#[derive(Debug, Serialize, Clone)]
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

#[derive(Serialize, Debug, Clone)]
pub struct Session {
    pub day: u32,
    pub track: u32,
    pub time: (Time, Time),
    pub title: String,
    pub presenters: Vec<Presenter>,
    pub tags: Vec<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Period {
    pub time: (Time, Time),
    pub sessions: Vec<Session>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Agenda {
    pub date: String,
    pub periods: Vec<Period>,
}

impl Agenda {
    pub fn sessions(&self) -> Vec<&Session> {
        self.periods
            .iter()
            .map(|p| p.sessions.iter())
            .flatten()
            .collect()
    }
}

async fn extract_time_from_session_page<T: IntoUrl + Clone>(
    url: T,
) -> Result<(Time, Time), Box<dyn std::error::Error>> {
    let content = reqwest::get(url.clone()).await?.text().await?;
    let presneter = Html::parse_document(&content);

    let time = presneter
        .select(selector!(".session__time"))
        .next()
        .unwrap()
        .text()
        .collect::<String>();
    let (_, from, to) = time.split(" - ").collect_tuple().unwrap();
    let (from, to) = [from, to]
        .into_iter()
        .map(|t| t.parse())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect_tuple()
        .unwrap();

    Ok((from, to))
}

pub async fn fetch() -> Result<Vec<Agenda>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let agendas = client
        .get("https://2022.tgdf.tw/agenda")
        .send()
        .await?
        .text()
        .await?;
    let agendas = Html::parse_document(&agendas);
    let mut agenda_storage = vec![];
    for (day, agenda) in agendas.select(selector!(".agenda")).enumerate() {
        let day = (day + 1) as u32;
        let date = agenda
            .select(selector!(".agenda__label"))
            .next()
            .unwrap()
            .text()
            .join(" ");
        let mut periods = vec![];
        for period in agenda.select(selector!(".agenda__row")) {
            let session_time = period
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
            let mut session_storage = vec![];
            for (i, sessions) in period.select(selector!(".agenda__sessions")).enumerate() {
                let track = (i + 1) as u32;
                for session in sessions.select(selector!(".session")) {
                    let tags = session
                        .select(selector!(".session__badge"))
                        .map(|v| v.text().collect::<String>())
                        .collect::<Vec<_>>();
                    let presenter_links = session.select(selector!("p")).next().map(|v| {
                        v.select(selector!("a"))
                            .map(|link| {
                                format!(
                                    "https://2022.tgdf.tw{}",
                                    link.value().attr("href").unwrap()
                                )
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
                    let session_url = format!("https://2022.tgdf.tw{}", session_url);
                    session_storage.push(Session {
                        title,
                        track,
                        tags,
                        presenters,
                        time: extract_time_from_session_page(&session_url).await?,
                        day,
                        description: fetch_session_description(session_url).await?,
                    });
                }
            }
            if session_storage.is_empty() {
                continue;
            }
            periods.push(Period {
                time: (time_from, time_to),
                sessions: session_storage,
            });
        }
        agenda_storage.push(Agenda { date, periods });
    }

    Ok(agenda_storage)
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
