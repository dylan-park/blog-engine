use anyhow::{Context, Result};
use rss::{
    extension::dublincore::DublinCoreExtensionBuilder,
    {Channel, ChannelBuilder, Item, ItemBuilder},
};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::{OnceCell, RwLock};
use tracing::info;
use uuid::Uuid;

const TITLE: &str = "Dylan Park | Blog";
const LINK: &str = "https://blog.dylan-park.com/";
const DESCRIPTION: &str = "A blog by Dylan Park";

#[derive(Clone)]
pub struct RssState {
    pub channel: Arc<Mutex<Channel>>,
}

static RSS_STATE: OnceCell<RwLock<RssState>> = OnceCell::const_new();

pub async fn init_rss_state() -> Result<()> {
    let rss_channel = initialize_feed("./static/feed.xml")
        .await
        .context("Unable to initialize feed")?;
    let state = RssState {
        channel: Arc::new(Mutex::new(rss_channel)),
    };
    RSS_STATE.set(RwLock::new(state)).ok();

    Ok(())
}

pub async fn initialize_feed(path: &str) -> Result<Channel> {
    if Path::new(&path).exists() {
        info!("Feed found on disk, reading...");
        let file = File::open(path).context("Error opening feed.xml")?;
        let reader = BufReader::new(file);
        let channel = Channel::read_from(reader).context("Error reading feed into Channel")?;
        info!("Feed successfully read from disk");

        Ok(channel)
    } else {
        info!("No feed found on disk, creating");
        let channel = create_feed().await.context("Error creating feed")?;
        write_channel(&channel, None)
            .await
            .context("Error writing channel")?;
        info!("Feed successfully created and written to disk");

        Ok(channel)
    }
}

pub async fn create_feed() -> Result<Channel> {
    Ok(ChannelBuilder::default()
        .title(TITLE)
        .link(LINK)
        .description(DESCRIPTION)
        .last_build_date(chrono::Utc::now().to_rfc2822())
        .build())
}

pub async fn create_item(
    title: String,
    description: Option<String>,
    link: Option<String>,
    identifier: Option<String>,
) -> Result<Item> {
    let mut binding = ItemBuilder::default();
    let mut builder = binding
        .title(Some(title))
        .guid(Some(rss::Guid {
            value: Uuid::new_v4().to_string(),
            permalink: false,
        }))
        .pub_date(Some(chrono::Utc::now().to_rfc2822()));

    if let Some(description) = description {
        builder = builder.description(Some(description));
    }
    if let Some(link) = link {
        builder = builder.link(Some(link));
    }
    if let Some(identifier) = identifier {
        builder = builder.dublin_core_ext(
            DublinCoreExtensionBuilder::default()
                .identifier(identifier)
                .build(),
        );
    }

    let item = builder.build();
    let item_clone = item.clone();
    info!(
        "Item Created:\n{{\n\t\"title\": \"{}\"\n\t\"description\": \"{}\"\n\t\"link\": \"{}\"\n}}",
        item_clone.title.unwrap(),
        item_clone.description.unwrap_or_default(),
        item_clone.link.unwrap_or_default()
    );

    Ok(item)
}

pub async fn write_channel(channel: &Channel, path: Option<&str>) -> Result<()> {
    let rss_content = channel.to_string();
    let file_path = path.unwrap_or("./static/feed.xml");
    fs::write(file_path, &rss_content).context("Error writing channel")?;
    info!("Feed written successfully");

    Ok(())
}

pub async fn add_item(state: RssState, item: Item) -> Result<()> {
    let mut channel = state.channel.lock().unwrap();
    let mut items = channel.items().to_vec();
    items.insert(0, item);
    channel.set_items(items);
    channel.set_last_build_date(chrono::Utc::now().to_rfc2822());

    // Save to file
    write_channel(&channel, None)
        .await
        .context("Unable to write to channel")?;

    Ok(())
}
