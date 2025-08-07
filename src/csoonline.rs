use chrono::{TimeZone};
use rss::{Item, ItemBuilder};
use scraper::{ElementRef, Html, Selector};

use crate::RSSSource;

pub struct CSOSource {
    prefix: String,
    document: Html,
}

pub struct CSOArticle<'a> {
    el: ElementRef<'a>,
}

impl CSOSource {
    pub fn default() -> CSOSource {
        CSOSource {
            prefix: "https://www.csoonline.com/privacy".to_string(),
            document: Html::new_document()
        }
    }

    async fn get_document(&self) -> Result<Html, Box<dyn std::error::Error>> {
        let page = reqwest::get(self.prefix.clone())
            .await?
            .text()
            .await?;

        Ok(Html::parse_document(&page))
    }

    async fn get_articles(&mut self) -> Result<Vec<CSOArticle<'_>>, Box<dyn std::error::Error>> {
        self.document = self.get_document().await?;
        let articles_selector = Selector::parse(".latest-content__content-featured, .latest-content__card-main, #article .content-listing-articles__row").unwrap();

        let articles = self.document.select(&articles_selector).map(|value| CSOArticle{el: value}).collect();
        Ok(articles)
    }
}

impl RSSSource for CSOSource {
    async fn get(&mut self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        Ok(self.get_articles()
            .await?
            .iter()
            .map(|el| el.into())
            .collect())
    }
}

impl Into<Item> for &CSOArticle<'_> {
    fn into(self) -> Item {
        let title_selector = Selector::parse("h3").unwrap();
        let title: String = self.el.select(&title_selector).next().unwrap().text().next().unwrap().into();

        let link_selector = Selector::parse("a").unwrap();
        let link: String = self.el.select(&link_selector).next().unwrap().attr("href").unwrap().into();

        let date_selector = Selector::parse(".card__info.card__info--light span").unwrap();
        let date_raw = self.el.select(&date_selector).next().unwrap().text().next().unwrap().trim();

        let parsed_date = chrono::NaiveDate::parse_from_str(date_raw, "%b %d, %Y")
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
