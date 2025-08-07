use chrono::DateTime;
use rss::{ChannelBuilder, Item};

use crate::{csoonline::CSOSource, darkreading::DarkReadingSource, krebs::KrebsSource};

mod krebs;
mod darkreading;
mod csoonline;

trait RSSSource {
    async fn get(&mut self) -> Result<Vec<Item>, Box<dyn std::error::Error>>;
}

async fn get_channel() -> Result<rss::Channel, Box<dyn std::error::Error>> {
    let mut cso_source = CSOSource::default();
    let mut krebs_source = KrebsSource::default();
    let mut darkreading_source = DarkReadingSource::default();

    let (cso, krebs, darkreading) = tokio::join!(
        cso_source.get(),
        krebs_source.get(),
        darkreading_source.get()
    );

    let cso = cso?;
    let krebs = krebs?;
    let darkreading = darkreading?;

    let mut items: Vec<Item> = darkreading.into_iter()
        .chain(cso.into_iter())
        .chain(krebs.into_iter())
        .collect();

    items.sort_by_key(|el| DateTime::parse_from_rfc2822(el.pub_date().unwrap_or_default()).unwrap_or_default());
    items.reverse();

    let channel = ChannelBuilder::default()
        .title("Rustfully syndicated")
        .items(items)
        .build();

    Ok(channel)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = get_channel().await?;

    println!("{channel}");
    Ok(())
}
