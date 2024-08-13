use crate::{modules::formatters::gog_formatter, modules::helpers::format_name, service::torrent::{Torrent, TorrentProvider}};
use async_trait::async_trait;
use fake_user_agent::get_rua;
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::Arc;
use crate::prisma::PrismaClient;

pub struct ProviderGOG {
    prisma_client: Arc<PrismaClient>,
    client: Client,
}

impl ProviderGOG {
    pub fn new(prisma_client: Arc<PrismaClient>) -> Self {
        ProviderGOG {
            prisma_client,
            client: Client::new(),
        }
    }

    pub async fn init_scraping(&self) -> Result<(), String> {
        match self.process_page().await {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Error during scraping: {}", e);
                Ok(())
            }
        }
    }

    async fn process_page(&self) -> Result<(), String> {
        let url = "https://freegogpcgames.com/a-z-games-list/";
        match self.fetch_web_content(url).await {
            Ok(data) => {
                if data.len() < 100 {
                    return Ok(());
                }

                let document = Html::parse_document(&data);
                let title_selector = Selector::parse(".items-inner > .letter-section > .az-columns > li > a").unwrap();

                for element in document.select(&title_selector) {
                    let title = element.text().collect::<Vec<_>>().join("");
                    let formatted_title = format_name(gog_formatter(title));
                    let link = match element.value().attr("href") {
                        Some(url) => url.to_string(),
                        None => continue,
                    };

                    let torrent = Torrent {
                        name: formatted_title,
                        repacker: "GOG".to_string(),
                        torrent: link,
                    };

                    let prisma_client = self.prisma_client.clone();
                    tokio::spawn(async move {
                        if let Err(e) = prisma_client
                            .torrent()
                            .create(torrent.name, torrent.repacker, torrent.torrent, vec![])
                            .exec()
                            .await {
                                println!("Ошибка при добавлении торрента: {}", e);
                            }
                    });
                }

                Ok(())
            }
            Err(error) => {
                println!("Ошибка при обработке страницы: {}", error);
                Err(error)
            }
        }
    }

    async fn fetch_web_content(&self, url: &str) -> Result<String, String> {
        let user_agent = get_rua();
        self.client
            .get(url)
            .header("User-Agent", user_agent)
            .send()
            .await
            .map_err(|e| format!("Ошибка HTTP запроса: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Ошибка чтения текста ответа: {}", e))
    }
}

#[async_trait]
impl TorrentProvider for ProviderGOG {
    async fn fetch_torrents(&self) -> Result<Vec<Torrent>, String> {
        self.init_scraping().await?;
        Ok(vec![])
    }
}
