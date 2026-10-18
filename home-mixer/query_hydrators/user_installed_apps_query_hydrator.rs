use crate::models::query::ScoredPostsQuery;
use crate::params::{EnableUserInstalledAppsHydration, EnableUserInstalledAppsV2};
use std::sync::Arc;
use std::time::Duration;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::user_installed_apps_store_client::UserInstalledAppsStoreClient;
use xai_candidate_pipeline::query_hydrator::QueryHydrator;
use xai_recsys_proto::installed_apps::{InstalledApp, NUM_INSTALLED_APPS, NUM_INSTALLED_APPS_128};

const MH_GET_TIMEOUT: Duration = Duration::from_millis(300);

pub struct UserInstalledAppsQueryHydrator {
    client: Arc<dyn UserInstalledAppsStoreClient>,
}

impl UserInstalledAppsQueryHydrator {
    pub fn new(client: Arc<dyn UserInstalledAppsStoreClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl QueryHydrator<ScoredPostsQuery> for UserInstalledAppsQueryHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableUserInstalledAppsHydration) || query.is_shadow_traffic
    }

    async fn hydrate(&self, query: &ScoredPostsQuery) -> Result<ScoredPostsQuery, String> {
        let slugs = tokio::time::timeout(MH_GET_TIMEOUT, self.client.fetch(query.user_id))
            .await
            .map_err(|_| "UserInstalledApps MH get timed out".to_string())??;

        let installed_apps = slugs.as_ref().map(|s| {
            if query.params.get(EnableUserInstalledAppsV2) {
                InstalledApp::build_multi_hot(s, NUM_INSTALLED_APPS_128)
            } else {
                InstalledApp::build_multi_hot(s, NUM_INSTALLED_APPS)
            }
        });

        Ok(ScoredPostsQuery {
            user_installed_apps: installed_apps,
            ..Default::default()
        })
    }

    fn update(&self, query: &mut ScoredPostsQuery, hydrated: ScoredPostsQuery) {
        query.user_installed_apps = hydrated.user_installed_apps;
    }
}
