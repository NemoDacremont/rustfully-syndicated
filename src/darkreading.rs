use chrono::{TimeZone};
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use rss::{Item, ItemBuilder};
use scraper::{ElementRef, Html, Selector};

use crate::RSSSource;

pub struct DarkReadingSource {
    prefix: String,
}

pub struct DarkReadingArticle<'a> {
    prefix: String,
    el: ElementRef<'a>,
}

impl DarkReadingSource {
    pub fn default() -> DarkReadingSource {
        DarkReadingSource {
            prefix: "https://www.darkreading.com/cyber-risk/data-privacy".to_string(),
        }
    }
}

impl RSSSource for DarkReadingSource {
    async fn get(&self) -> Result<Vec<Item>, anyhow::Error> {
        let req = reqwest::Client::new().get(self.prefix.clone())
            .header(USER_AGENT, "Mozilla/5.0 (X11; U; Linux x86_64; en-ca) AppleWebKit/531.2+ (KHTML, like Gecko) Version/5.0 Safari/531.2+")
            .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.5")
            .header("Sec-GPC", "1");

        let res = req.send().await?;
        let page = res.text().await?;

        let document = Html::parse_document(&page);

        let articles_selector = Selector::parse(".ListContent-Body .ContentPreview").unwrap();
        let articles = document.select(&articles_selector).map(|value| DarkReadingArticle{el: value, prefix: self.prefix.to_owned()});

        let items: Vec<Item> = articles.map(|el| el.into()).collect();
        Ok(items)
    }
}

impl Into<Item> for DarkReadingArticle<'_> {
    fn into(self) -> Item {
        let title_selector = Selector::parse(".ArticlePreview-Title, .ListPreview-Title, .ContentCard-Title").unwrap();
        let title: String = self.el.select(&title_selector).next().unwrap().text().next().unwrap().into();
        let link_suffix: String = self.el.select(&title_selector).next().unwrap().attr("href").unwrap().into();
        let mut link = self.prefix;
        link.push_str(&link_suffix);

        let date_selector = Selector::parse(".ArticlePreview-Date, .ContentCard-Date, .ListPreview-Date").unwrap();
        let date_raw = self.el.select(&date_selector).next().unwrap().text().next().unwrap();

        let parsed_date = chrono::NaiveDate::parse_from_str(date_raw, "%b %d, %Y")
            .expect("Failed to parse date");

        let dt = chrono::Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap());
        let date = dt.to_rfc2822();

        let item = ItemBuilder::default()
            .title(title)
            .link(link)
            .pub_date(date)
            .build();

        return item;
    }
}
