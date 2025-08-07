use chrono::{TimeZone};
use rss::{Item, ItemBuilder};
use scraper::{ElementRef, Html, Selector};

use crate::RSSSource;

pub struct KrebsSource {
    prefix: String,
    document: Html,
}

pub struct KrebsArticle<'a> {
    el: ElementRef<'a>,
}

impl KrebsSource {
    pub fn default() -> KrebsSource {
        KrebsSource { prefix: "https://krebsonsecurity.com".to_string(), document: Html::new_document() }
    }

    async fn get_document(&self) -> Result<Html, Box<dyn std::error::Error>> {
        let page = reqwest::get(self.prefix.clone())
            .await?
            .text()
            .await?;

        Ok(Html::parse_document(&page))
    }

    async fn get_articles(&mut self) -> Result<Vec<KrebsArticle<'_>>, Box<dyn std::error::Error>> {
        self.document = self.get_document().await?;
        let articles_selector = Selector::parse("#primary article").unwrap();

        let articles = self.document.select(&articles_selector).map(|value| KrebsArticle{el: value}).collect();
        Ok(articles)
    }
}

impl RSSSource for KrebsSource {
    async fn get(&mut self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        Ok(self.get_articles()
            .await?
            .iter()
            .map(|el| el.into())
            .collect())
    }
}

impl Into<Item> for &KrebsArticle<'_> {
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
