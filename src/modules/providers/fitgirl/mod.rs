use crate::{modules::formatters::fitgirl_formatter, modules::helpers::format_name, service::torrent::{Torrent, TorrentProvider}};
use async_trait::async_trait;
use fake_user_agent::get_rua;
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::prisma::PrismaClient;

pub struct ProviderFitGirl {
    prisma_client: Arc<PrismaClient>,
    client: Client,
    total_pages: Arc<Mutex<u32>>,
    processed_pages: Arc<Mutex<u32>>,
    max_page_in_queue: Arc<Mutex<u32>>,
}

impl ProviderFitGirl {
    pub fn new(prisma_client: Arc<PrismaClient>) -> Self {
        ProviderFitGirl {
            prisma_client,
            client: Client::new(),
            total_pages: Arc::new(Mutex::new(0)),
            processed_pages: Arc::new(Mutex::new(0)),
            max_page_in_queue: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn init_scraping(&self) -> Result<(), String> {
        match self.get_total_pages().await {
            Ok(total_pages) => {
                *self.total_pages.lock().unwrap() = total_pages;
                self.collect_pages(total_pages).await;
                Ok(())
            }
            Err(e) => {
                println!("Error getting total pages: {}", e);
                Ok(())
            }
        }
    }

    async fn get_total_pages(&self) -> Result<u32, String> {
        let url = "https://www.1337xx.to/user/FitGirl/1";
        let text = self.fetch_web_content(url).await?;
        let document = Html::parse_document(&text);
        let selector = Selector::parse(".pagination > ul > li:last-child > a").unwrap();

        let total_pages = document
            .select(&selector)
            .next()
            .and_then(|element| element.text().next())
            .and_then(|text| text.parse::<u32>().ok())
            .unwrap_or(1);

        Ok(total_pages)
    }

    async fn collect_pages(&self, up_to_page: u32) {
        let (tx, mut rx) = mpsc::channel(10);

        for page in (*self.max_page_in_queue.lock().unwrap() + 1)..=up_to_page {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                tx_clone.send(page).await.unwrap();
            });
        }

        *self.max_page_in_queue.lock().unwrap() = up_to_page;

        let prisma_client = self.prisma_client.clone();
        let client = self.client.clone();
        let total_pages = self.total_pages.clone();
        let processed_pages = self.processed_pages.clone();
        let max_page_in_queue = self.max_page_in_queue.clone();

        tokio::spawn(async move {
            while let Some(page) = rx.recv().await {
                let prisma_client_clone = prisma_client.clone();
                let client_clone = client.clone();
                let total_pages_clone = total_pages.clone();
                let processed_pages_clone = processed_pages.clone();
                let max_page_in_queue_clone = max_page_in_queue.clone();

                tokio::spawn(async move {
                    let provider = ProviderFitGirl {
                        prisma_client: prisma_client_clone,
                        client: client_clone,
                        total_pages: total_pages_clone,
                        processed_pages: processed_pages_clone,
                        max_page_in_queue: max_page_in_queue_clone,
                    };
                    match provider.process_page(page).await {
                        Ok(_) => {}
                        Err(e) => println!("Error processing page {}: {}", page, e),
                    }
                });
            }
        });
    }

    async fn process_page(&self, page: u32) -> Result<(), String> {
        let url = format!("https://www.1337xx.to/user/FitGirl/{}", page);
        match self.fetch_web_content(&url).await {
            Ok(data) => {
                if data.len() < 100 {
                    return Ok(());
                }

                let document = Html::parse_document(&data);
                let title_selector = Selector::parse(".table-list tbody tr td.coll-1.name a[href]:nth-of-type(2)").unwrap();

                for element in document.select(&title_selector) {
                    let title = element.text().collect::<Vec<_>>().join("");
                    let formatted_title = format_name(fitgirl_formatter(title));
                    let link = match element.value().attr("href") {
                        Some(url) => url.to_string(),
                        None => continue,
                    };

                    let torrent = Torrent {
                        name: formatted_title,
                        repacker: "FitGirl".to_string(),
                        torrent: format!("https://www.1337xx.to{}", link),
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

                *self.processed_pages.lock().unwrap() += 1;
                Ok(())
            }
            Err(error) => {
                println!("Ошибка при обработке страницы {}: {}", page, error);
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
impl TorrentProvider for ProviderFitGirl {
    async fn fetch_torrents(&self) -> Result<Vec<Torrent>, String> {
        self.init_scraping().await?;
        Ok(vec![])
    }
}
