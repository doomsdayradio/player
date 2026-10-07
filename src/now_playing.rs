use eframe::egui;
use reqwest::header::{HeaderValue, AUTHORIZATION};
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

const DEFAULT_SERVER_URL: &str = "https://ntfy.bot.omg.lol";
const DEFAULT_TOPIC: &str = "doomsday";
const MAX_LINE_BYTES: usize = 64 * 1024;

struct Configuration {
    url: reqwest::Url,
    topic: String,
}

impl Configuration {
    fn from_env() -> Result<Self, String> {
        let server = Self::setting(
            std::env::var("NTFY_URL").ok(),
            option_env!("NTFY_URL"),
            DEFAULT_SERVER_URL,
        );
        let topic = Self::setting(
            std::env::var("NTFY_TOPIC").ok(),
            option_env!("NTFY_TOPIC"),
            DEFAULT_TOPIC,
        );
        Self::parse(&server, &topic)
    }

    fn setting(runtime: Option<String>, build: Option<&str>, fallback: &str) -> String {
        runtime
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                build
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| fallback.to_owned())
    }

    fn token(
        runtime: Option<String>,
        legacy: Option<String>,
        embedded: Option<&str>,
    ) -> Option<String> {
        runtime
            .filter(|value| !value.trim().is_empty())
            .or_else(|| legacy.filter(|value| !value.trim().is_empty()))
            .or_else(|| {
                embedded
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_owned)
            })
    }

    fn parse(server: &str, topic: &str) -> Result<Self, String> {
        let mut url =
            reqwest::Url::parse(server.trim()).map_err(|_| "NTFY_URL ist ungültig".to_string())?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("NTFY_URL muss eine HTTPS-Serveradresse ohne Zugangsdaten sein".into());
        }
        let topic = topic.trim();
        if topic.is_empty()
            || topic.len() > 64
            || !topic
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err("NTFY_TOPIC ist ungültig".into());
        }
        url.path_segments_mut()
            .map_err(|_| "NTFY_URL ist ungültig".to_string())?
            .pop_if_empty()
            .push(topic)
            .push("json");
        Ok(Self {
            url,
            topic: topic.to_owned(),
        })
    }
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub message: Option<String>,
    pub error: Option<String>,
}

pub struct NowPlaying {
    snapshot: Arc<Mutex<Snapshot>>,
    cancelled: Arc<AtomicBool>,
    worker: thread::JoinHandle<()>,
}

impl NowPlaying {
    pub fn new(context: egui::Context) -> Self {
        let snapshot = Arc::new(Mutex::new(Snapshot::default()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_snapshot = snapshot.clone();
        let worker_cancelled = cancelled.clone();
        let worker = thread::spawn(move || {
            let result = receive(&worker_snapshot, &worker_cancelled, &context);
            if let Err(error) = result {
                worker_snapshot.lock().unwrap().error = Some(error);
                context.request_repaint();
            }
        });
        Self {
            snapshot,
            cancelled,
            worker,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.lock().unwrap().clone()
    }
}

impl Drop for NowPlaying {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        self.worker.thread().unpark();
    }
}

#[derive(Default)]
struct Latest {
    topic: String,
    time: u64,
    ids: HashSet<String>,
    cursor: Option<String>,
}

impl Latest {
    fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_owned(),
            ..Default::default()
        }
    }

    fn accept(&mut self, line: &[u8]) -> Result<Option<String>, String> {
        let event: serde_json::Value = serde_json::from_slice(line)
            .map_err(|_| "Ungültige Titelmeldung vom Server".to_string())?;
        if event["event"] != "message" || event["topic"].as_str() != Some(self.topic.as_str()) {
            return Ok(None);
        }
        let time = event["time"]
            .as_u64()
            .ok_or("Titelmeldung ohne Zeitstempel")?;
        let id = event["id"].as_str().ok_or("Titelmeldung ohne ID")?;
        let message = event["message"].as_str().ok_or("Titelmeldung ohne Text")?;
        if time < self.time || (time == self.time && self.ids.contains(id)) {
            return Ok(None);
        }
        if time > self.time {
            self.ids.clear();
        }
        self.time = time;
        self.ids.insert(id.to_owned());
        self.cursor = Some(id.to_owned());
        Ok(Some(message.trim().to_owned()))
    }
}

async fn cancelled(flag: &AtomicBool) {
    while !flag.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn process_chunk(
    chunk: &[u8],
    pending: &mut Vec<u8>,
    latest: &mut Latest,
    mut publish: impl FnMut(String),
) -> Result<(), String> {
    for &byte in chunk {
        if byte == b'\n' {
            if !pending.is_empty() {
                if let Some(message) = latest.accept(pending)? {
                    publish(message);
                }
                pending.clear();
            }
        } else {
            if pending.len() >= MAX_LINE_BYTES {
                return Err("Titelmeldung ist zu groß".into());
            }
            pending.push(byte);
        }
    }
    Ok(())
}

fn client() -> Result<reqwest::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(token) = Configuration::token(
        std::env::var("NTFY_TOKEN").ok(),
        std::env::var("DOOMSDAY_NTFY_TOKEN").ok(),
        option_env!("DOOMSDAY_EMBEDDED_NTFY_TOKEN"),
    ) {
        let mut value = HeaderValue::from_str(&format!("Bearer {}", token.trim()))
            .map_err(|_| "NTFY_TOKEN ist ungültig".to_string())?;
        value.set_sensitive(true);
        headers.insert(AUTHORIZATION, value);
    }
    reqwest::Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(8))
        .read_timeout(Duration::from_secs(75))
        .user_agent(concat!("DoomsdayRadio/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| "Titelabfrage konnte nicht gestartet werden".to_string())
}

fn receive(
    snapshot: &Mutex<Snapshot>,
    flag: &AtomicBool,
    context: &egui::Context,
) -> Result<(), String> {
    let configuration = Configuration::from_env()?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Titelabfrage konnte nicht gestartet werden".to_string())?;
    let client = client()?;
    let mut latest = Latest::new(&configuration.topic);
    while !flag.load(Ordering::Relaxed) {
        let result = runtime.block_on(async {
            tokio::select! {
                _ = cancelled(flag) => Ok(()),
                result = listen(&client, &configuration.url, &mut latest, snapshot, context) => result,
            }
        });
        if flag.load(Ordering::Relaxed) {
            break;
        }
        snapshot.lock().unwrap().error = Some(
            result
                .err()
                .unwrap_or_else(|| "Titelverbindung unterbrochen".into()),
        );
        context.request_repaint();
        thread::park_timeout(Duration::from_secs(10));
    }
    Ok(())
}

async fn listen(
    client: &reqwest::Client,
    url: &reqwest::Url,
    latest: &mut Latest,
    snapshot: &Mutex<Snapshot>,
    context: &egui::Context,
) -> Result<(), String> {
    let mut response = client
        .get(url.clone())
        .query(&[("since", latest.cursor.as_deref().unwrap_or("all"))])
        .send()
        .await
        .map_err(|_| "Titelserver nicht erreichbar".to_string())?;
    match response.status().as_u16() {
        200 => {}
        401 | 403 => {
            return Err("Titelzugriff verweigert: NTFY_TOKEN prüfen".into());
        }
        400 if latest.cursor.is_some() => {
            latest.cursor = None;
            return Err("Titelverbindung wird neu aufgebaut".into());
        }
        _ => return Err(format!("Titelserver: HTTP {}", response.status().as_u16())),
    }
    snapshot.lock().unwrap().error = None;
    context.request_repaint();
    let mut pending = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Titelverbindung unterbrochen".to_string())?
    {
        process_chunk(&chunk, &mut pending, latest, |message| {
            snapshot.lock().unwrap().message = Some(message);
            context.request_repaint();
        })?;
    }
    Err("Titelverbindung unterbrochen".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_tokens_override_the_embedded_read_token() {
        assert_eq!(
            Configuration::token(
                Some("local".into()),
                Some("legacy".into()),
                Some("embedded")
            ),
            Some("local".into())
        );
        assert_eq!(
            Configuration::token(None, Some("legacy".into()), Some("embedded")),
            Some("legacy".into())
        );
        assert_eq!(
            Configuration::token(None, None, Some("embedded")),
            Some("embedded".into())
        );
    }

    #[test]
    fn empty_tokens_fall_back_or_allow_anonymous_access() {
        assert_eq!(
            Configuration::token(Some(" ".into()), Some("".into()), Some("embedded")),
            Some("embedded".into())
        );
        assert_eq!(Configuration::token(None, None, Some(" ")), None);
        assert_eq!(Configuration::token(None, None, None), None);
    }

    fn message(id: &str, time: u64, text: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "event": "message", "topic": "doomsday", "id": id,
            "time": time, "message": text,
        }))
        .unwrap()
    }

    #[test]
    fn newest_cached_message_wins_even_when_arriving_out_of_order() {
        let mut latest = Latest::new("doomsday");
        assert_eq!(
            latest.accept(&message("new", 20, "Artist - Song")).unwrap(),
            Some("Artist - Song".into())
        );
        assert_eq!(
            latest.accept(&message("old", 10, "Old Song")).unwrap(),
            None
        );
        assert_eq!(latest.cursor.as_deref(), Some("new"));
    }

    #[test]
    fn same_second_updates_work_without_replaying_duplicates() {
        let mut latest = Latest::new("doomsday");
        latest.accept(&message("first", 20, "First Song")).unwrap();
        assert_eq!(
            latest
                .accept(&message("second", 20, "Second Song"))
                .unwrap(),
            Some("Second Song".into())
        );
        assert_eq!(
            latest.accept(&message("first", 20, "First Song")).unwrap(),
            None
        );
        assert_eq!(latest.cursor.as_deref(), Some("second"));
    }

    #[test]
    fn connection_events_and_other_topics_do_not_replace_the_title() {
        let mut latest = Latest::new("doomsday");
        for event in [
            br#"{"event":"open"}"#.as_slice(),
            br#"{"event":"keepalive"}"#.as_slice(),
            br#"{"event":"message","topic":"other"}"#.as_slice(),
        ] {
            assert_eq!(latest.accept(event).unwrap(), None);
        }
        assert!(latest.cursor.is_none());
    }

    #[test]
    fn malformed_messages_do_not_advance_the_cursor() {
        let mut latest = Latest::new("doomsday");
        assert!(latest.accept(b"not json").is_err());
        assert!(latest
            .accept(br#"{"event":"message","topic":"doomsday"}"#)
            .is_err());
        assert!(latest.cursor.is_none());
    }

    #[test]
    fn unicode_and_multiline_titles_are_preserved() {
        let mut latest = Latest::new("doomsday");
        assert_eq!(
            latest
                .accept(&message("title", 30, "  Künstler\nNächster Titel  "))
                .unwrap(),
            Some("Künstler\nNächster Titel".into())
        );
    }

    #[test]
    fn fragmented_stream_reassembles_unicode_and_multiple_messages() {
        let mut stream = b"{\"event\":\"open\"}\n".to_vec();
        stream.extend(message("first", 10, "Künstler - Erster Titel"));
        stream.push(b'\n');
        stream.extend(message("second", 20, "Nächster Titel"));
        stream.push(b'\n');
        let mut latest = Latest::new("doomsday");
        let mut pending = Vec::new();
        let mut titles = Vec::new();
        for chunk in stream.chunks(3) {
            process_chunk(chunk, &mut pending, &mut latest, |title| titles.push(title)).unwrap();
        }
        assert_eq!(titles, ["Künstler - Erster Titel", "Nächster Titel"]);
        assert!(pending.is_empty());
        assert_eq!(latest.cursor.as_deref(), Some("second"));
    }

    #[test]
    fn oversized_unterminated_messages_are_rejected() {
        let mut latest = Latest::new("doomsday");
        let mut pending = Vec::new();
        assert!(process_chunk(
            &vec![b'x'; MAX_LINE_BYTES + 1],
            &mut pending,
            &mut latest,
            |_| panic!("Invalid message must not be published"),
        )
        .is_err());
        assert_eq!(pending.len(), MAX_LINE_BYTES);
        assert!(latest.cursor.is_none());
    }

    #[test]
    fn server_and_topic_form_the_stream_url() {
        for server in ["https://ntfy.bot.omg.lol", "https://ntfy.bot.omg.lol/"] {
            let configuration = Configuration::parse(server, "doomsday").unwrap();
            assert_eq!(
                configuration.url.as_str(),
                "https://ntfy.bot.omg.lol/doomsday/json"
            );
        }
        let configuration =
            Configuration::parse("https://example.com/ntfy/", "music-live").unwrap();
        assert_eq!(
            configuration.url.as_str(),
            "https://example.com/ntfy/music-live/json"
        );
        assert_eq!(configuration.topic, "music-live");
    }

    #[test]
    fn unsafe_urls_and_invalid_topics_are_rejected() {
        for server in [
            "http://example.com",
            "not a url",
            "https://user:password@example.com",
            "https://example.com?token=value",
            "https://example.com#fragment",
        ] {
            assert!(Configuration::parse(server, "doomsday").is_err());
        }
        for topic in ["", "other/topic", "..", "topic?query", "two topics"] {
            assert!(Configuration::parse(DEFAULT_SERVER_URL, topic).is_err());
        }
    }

    #[test]
    fn configured_topic_is_used_to_filter_messages() {
        let mut latest = Latest::new("music-live");
        assert_eq!(
            latest.accept(&message("other", 20, "Other Song")).unwrap(),
            None
        );
        let event = serde_json::to_vec(&serde_json::json!({
            "event": "message", "topic": "music-live", "id": "new",
            "time": 30, "message": "Current Song",
        }))
        .unwrap();
        assert_eq!(latest.accept(&event).unwrap(), Some("Current Song".into()));
    }

    #[test]
    fn runtime_settings_override_build_defaults_and_empty_values_fall_back() {
        assert_eq!(
            Configuration::setting(Some("local".into()), Some("build"), "default"),
            "local"
        );
        assert_eq!(
            Configuration::setting(None, Some("build"), "default"),
            "build"
        );
        assert_eq!(
            Configuration::setting(Some(" ".into()), Some("build"), "default"),
            "build"
        );
        assert_eq!(
            Configuration::setting(Some("".into()), Some(" "), "default"),
            "default"
        );
    }

    #[test]
    #[ignore = "Requires live access to the configured ntfy topic"]
    fn live_ntfy_topic_is_readable() {
        let configuration = Configuration::from_env().unwrap();
        let client = client().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(20), async {
                let mut response = client
                    .get(configuration.url)
                    .query(&[("poll", "1"), ("since", "all")])
                    .send()
                    .await
                    .map_err(|_| "ntfy server unavailable".to_string())?;
                if response.status().as_u16() != 200 {
                    return Err(format!("ntfy returned HTTP {}", response.status().as_u16()));
                }
                let mut latest = Latest::new(&configuration.topic);
                let mut pending = Vec::new();
                while let Some(chunk) = response
                    .chunk()
                    .await
                    .map_err(|_| "ntfy response interrupted".to_string())?
                {
                    process_chunk(&chunk, &mut pending, &mut latest, |_| {})?;
                }
                if !pending.is_empty() {
                    return Err("ntfy response contains an incomplete message".into());
                }
                Ok::<(), String>(())
            })
            .await
            .map_err(|_| "ntfy request timed out".to_string())?
        });
        assert!(
            result.is_ok(),
            "ntfy access check failed: {}",
            result.unwrap_err()
        );
    }
}
