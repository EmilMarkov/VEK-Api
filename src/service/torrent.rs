use crate::prisma::PrismaClient;
use crate::prisma::torrent;
use std::sync::Arc;

use crate::modules::providers::dodi::ProviderDODI;
use crate::modules::providers::emperss::Provider0xEMPRESS;
use crate::modules::providers::fitgirl::ProviderFitGirl;
use crate::modules::providers::gog::ProviderGOG;
use crate::modules::providers::kaoskrew::ProviderKaOsKrew;
use crate::modules::providers::onlinefix::ProviderOnlineFix;
use crate::modules::providers::tinyrepacks::ProviderTinyRepacks;
use crate::modules::providers::xatab::ProviderXatab;

pub struct TorrentService {
    prisma_client: Arc<PrismaClient>,
}

impl TorrentService {
    pub fn new(prisma_client: Arc<PrismaClient>) -> Self {
        TorrentService { prisma_client }
    }

    pub async fn initialize_torrents(&self) -> Result<(), String> {
        let providers: Vec<(&str, Box<dyn TorrentProvider>)> = vec![
            ("DODI", Box::new(ProviderDODI::new(self.prisma_client.clone()))),
            ("FitGirl", Box::new(ProviderFitGirl::new(self.prisma_client.clone()))),
            ("0xEMPRESS", Box::new(Provider0xEMPRESS::new(self.prisma_client.clone()))),
            ("GOG", Box::new(ProviderGOG::new(self.prisma_client.clone()))),
            ("KaOsKrew", Box::new(ProviderKaOsKrew::new(self.prisma_client.clone()))),
            ("OnlineFix", Box::new(ProviderOnlineFix::new(self.prisma_client.clone()))),
            ("TinyRepacks", Box::new(ProviderTinyRepacks::new(self.prisma_client.clone()))),
            ("Xatab", Box::new(ProviderXatab::new(self.prisma_client.clone()))),
        ];

        for (name, provider) in providers {
            match provider.fetch_torrents().await {
                Ok(torrents) => {
                    for torrent in torrents {
                        let _ = self.prisma_client
                            .torrent()
                            .create(torrent.name, torrent.repacker, torrent.torrent, vec![])
                            .exec()
                            .await;
                    }
                }
                Err(_) => {}
            }
            println!("Завершен парсинг провайдера: {}", name);
        }

        Ok(())
    }

    pub async fn search_torrent(&self, game_name: &str) -> Result<Vec<(String, String)>, String> {
        let torrents = self
            .prisma_client
            .torrent()
            .find_many(vec![torrent::name::contains(game_name.to_string())])
            .exec()
            .await
            .map_err(|e| format!("Failed to search torrent: {}", e))?;

        Ok(torrents.into_iter().map(|t| (t.repacker, t.torrent)).collect())
    }
}

#[async_trait::async_trait]
pub trait TorrentProvider {
    async fn fetch_torrents(&self) -> Result<Vec<Torrent>, String>;
}

pub struct Torrent {
    pub name: String,
    pub repacker: String,
    pub torrent: String,
}
