use anyhow::{Result, bail};
use chrono::NaiveDateTime;

const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

/// 一篇 content/*.md
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub time: NaiveDateTime,
    pub title: String,
    pub body: String,
    pub data_rec: u8,
}

impl Entry {
    /// 解析 Telegram 訊息：
    ///
    /// ```text
    /// [YYYY-MM-DD HH:MM]   ← 選填，沒有就用訊息時間
    /// 標題
    /// 內文（選填，Markdown）
    /// ```
    pub fn parse(text: &str, now: NaiveDateTime) -> Result<Self> {
        let mut lines = text.lines().skip_while(|l| l.trim().is_empty());

        let Some(mut first) = lines.next() else {
            bail!("訊息是空的");
        };

        let time = match NaiveDateTime::parse_from_str(first.trim(), TIME_FORMAT) {
            Ok(t) => {
                let Some(next) = lines.by_ref().find(|l| !l.trim().is_empty()) else {
                    bail!("時間後面缺少標題");
                };
                first = next;
                t
            }
            Err(_) => now,
        };

        let body = lines
            .skip_while(|l| l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(Self {
            time,
            title: first.trim().to_owned(),
            body: body.trim_end().to_owned(),
            data_rec: fastrand::u8(10..=99),
        })
    }

    /// 檔名主幹，跟 scripts/new-event.ts 一樣是 `YYYY-MM-DD-HHmm`
    pub fn file_stem(&self) -> String {
        self.time.format("%Y-%m-%d-%H%M").to_string()
    }

    pub fn render(&self) -> String {
        format!(
            "---\nyear: \"{}\"\ntitle: \"{}\"\ndataRec: \"{}%\"\n---\n{}\n",
            self.time.format(TIME_FORMAT),
            escape_yaml(&self.title),
            self.data_rec,
            self.body,
        )
    }
}

/// YAML 雙引號字串跳脫
fn escape_yaml(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2026-10-14 16:22", TIME_FORMAT).unwrap()
    }

    #[test]
    fn title_only_uses_message_time() {
        let e = Entry::parse("Steam Frame!!!!", now()).unwrap();
        assert_eq!(e.time, now());
        assert_eq!(e.title, "Steam Frame!!!!");
        assert_eq!(e.body, "");
        assert_eq!(e.file_stem(), "2026-10-14-1622");
    }

    #[test]
    fn title_and_multiline_body() {
        let e = Entry::parse("\n  標題  \n第一行\n\n第三行\n\n", now()).unwrap();
        assert_eq!(e.title, "標題");
        assert_eq!(e.body, "第一行\n\n第三行");
    }

    #[test]
    fn leading_time_overrides_message_time() {
        let e = Entry::parse("2026-09-06 18:56\n是你嗎 瑪格麗特\n內文", now()).unwrap();
        assert_eq!(e.time.format(TIME_FORMAT).to_string(), "2026-09-06 18:56");
        assert_eq!(e.title, "是你嗎 瑪格麗特");
        assert_eq!(e.body, "內文");
        assert_eq!(e.file_stem(), "2026-09-06-1856");
    }

    #[test]
    fn time_without_title_is_rejected() {
        assert!(Entry::parse("2026-09-06 18:56\n\n", now()).is_err());
        assert!(Entry::parse("  \n ", now()).is_err());
    }

    #[test]
    fn data_rec_in_range() {
        for _ in 0..200 {
            let e = Entry::parse("t", now()).unwrap();
            assert!((10..=99).contains(&e.data_rec));
        }
    }

    #[test]
    fn render_matches_existing_format() {
        let e = Entry {
            time: now(),
            title: r#"say "hi" \o/"#.to_owned(),
            body: "Let's Go!!!".to_owned(),
            data_rec: 52,
        };
        assert_eq!(
            e.render(),
            "---\nyear: \"2026-10-14 16:22\"\ntitle: \"say \\\"hi\\\" \\\\o/\"\ndataRec: \"52%\"\n---\nLet's Go!!!\n"
        );
    }
}
