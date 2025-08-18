use chrono::{TimeZone};
use rss::{Item, ItemBuilder};
use scraper::{ElementRef, Html, Selector};

use crate::RSSSource;

pub struct KrebsSource {
    prefix: String,
}

pub struct KrebsArticle<'a> {
    el: ElementRef<'a>,
}

impl KrebsSource {
    pub fn default() -> KrebsSource {
        KrebsSource { prefix: "https://krebsonsecurity.com".to_string() }
    }
}

impl RSSSource for KrebsSource {
    async fn get(&self) -> Result<Vec<Item>, anyhow::Error> {
        let page = reqwest::get(self.prefix.clone())
            .await?
            .text()
            .await?;

        let document = Html::parse_document(&page);

        let articles_selector = Selector::parse("#primary article").unwrap();
        let articles = document.select(&articles_selector).map(|value| KrebsArticle{el: value});

        let items: Vec<Item> = articles
            .map(|el| el.into())
            .collect();

        Ok(items)
    }
}

impl Into<Item> for KrebsArticle<'_> {
    fn into(self) -> Item {
        let title_selector = Selector::parse("h2 a").unwrap();
        let title: String = self.el.select(&title_selector).next().unwrap().text().next().unwrap().into();
        let link: String = self.el.select(&title_selector).next().unwrap().attr("href").unwrap().into();

        let date_selector = Selector::parse(".date").unwrap();
        let date_raw = self.el.select(&date_selector).next().unwrap().text().next().unwrap().trim();

        let parsed_date = chrono::NaiveDate::parse_from_str(date_raw, "%B %d, %Y")
            .expect("Failed to parse date");

        let dt = chrono::Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap());
        let date = dt.to_rfc2822();

        let item = ItemBuilder::default()
            .title(title)
            .link(link)
            .pub_date(date.to_string())
            .build();

        return item;
    }
}
