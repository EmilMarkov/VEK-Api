use crate::{modules::formatters::onlinefix_formatter, modules::helpers::format_name, service::torrent::{Torrent, TorrentProvider}};
use async_trait::async_trait;
use fake_user_agent::get_rua;
use reqwest::{Client, header::HeaderValue};
use scraper::{Html, Selector};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::prisma::PrismaClient;
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

pub struct ProviderOnlineFix {
    prisma_client: Arc<PrismaClient>,
    client: Client,
    total_pages: Arc<Mutex<u32>>,
    processed_pages: Arc<Mutex<u32>>,
    max_page_in_queue: Arc<Mutex<u32>>,
}

impl ProviderOnlineFix {
    pub fn new(prisma_client: Arc<PrismaClient>) -> Self {
        let cookie_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));
        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        ProviderOnlineFix {
            prisma_client,
            client,
            total_pages: Arc::new(Mutex::new(0)),
            processed_pages: Arc::new(Mutex::new(0)),
            max_page_in_queue: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn init_scraping(&self) -> Result<(), String> {
        self.authenticate().await?;
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

    async fn authenticate(&self) -> Result<(), String> {
        let pre_login_url = "https://online-fix.me/engine/ajax/authtoken.php";
        let login_url = "https://online-fix.me/";
        let username = "your_username"; // replace with actual username
        let password = "your_password"; // replace with actual password
        let user_agent = get_rua();

        let pre_login_res = self.client.get(pre_login_url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", login_url)
            .header("User-Agent", HeaderValue::from_str(&user_agent).unwrap())
            .send()
            .await
            .map_err(|e| format!("Failed to get auth token: {}", e))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse auth token response: {}", e))?;

        let field = pre_login_res["field"].as_str().ok_or("No field in auth token")?;
        let value = pre_login_res["value"].as_str().ok_or("No value in auth token")?;

        let mut params = vec![
            ("login_name", username),
            ("login_password", password),
            ("login", "submit"),
        ];
        params.push((field, value));

        self.client.post(login_url)
            .header("Referer", login_url)
            .header("Origin", login_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", HeaderValue::from_str(&user_agent).unwrap())
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to authenticate: {}", e))?;

        Ok(())
    }

    async fn get_total_pages(&self) -> Result<u32, String> {
        let url = "https://online-fix.me/page/1";
        let text = self.fetch_web_content(url, "https://online-fix.me/").await?;
        let document = Html::parse_document(&text);
        let selector = Selector::parse("nav.pagination.hide_onajax a:nth-last-of-type(2)").unwrap();

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
                    let provider = ProviderOnlineFix {
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
        let url = format!("https://online-fix.me/page/{}", page);
        match self.fetch_web_content(&url, "https://online-fix.me/").await {
            Ok(data) => {
                if data.len() < 100 {
                    return Ok(());
                }

                let document = Html::parse_document(&data);
                let link_selector = Selector::parse("article.news > .article.clr > .article-content > a").unwrap();
                let title_selector = Selector::parse("h2.title").unwrap();

                for element in document.select(&link_selector) {
                    let title_element = element.select(&title_selector).next();
                    let title = title_element.map(|e| e.text().collect::<Vec<_>>().join("")).unwrap_or_default();
                    let formatted_title = format_name(onlinefix_formatter(title));
                    let link = match element.value().attr("href") {
                        Some(url) => url.to_string(),
                        None => continue,
                    };

                    let torrent = Torrent {
                        name: formatted_title,
                        repacker: "Online-Fix".to_string(),
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

                *self.processed_pages.lock().unwrap() += 1;
                Ok(())
            }
            Err(error) => {
                println!("Ошибка при обработке страницы {}: {}", page, error);
                Err(error)
            }
        }
    }

    async fn fetch_web_content(&self, url: &str, referer: &str) -> Result<String, String> {
        let user_agent = get_rua();

        self.client.get(url)
            .header("User-Agent", HeaderValue::from_str(&user_agent).unwrap())
            .header("Referer", HeaderValue::from_str(referer).unwrap())
            .send()
            .await
            .map_err(|e| format!("Ошибка HTTP запроса: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Ошибка чтения текста ответа: {}", e))
    }
}

#[async_trait]
impl TorrentProvider for ProviderOnlineFix {
    async fn fetch_torrents(&self) -> Result<Vec<Torrent>, String> {
        self.init_scraping().await?;
        Ok(vec![])
    }
}
