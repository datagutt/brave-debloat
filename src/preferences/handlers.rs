use super::types::*;

pub fn get_default_search_provider(prefs_config: Option<&PreferencesInputConfig>) -> SearchProvider {
    prefs_config
        .and_then(|p| p.search_engines.first())
        .cloned()
        .unwrap_or_else(|| SearchProvider {
            keyword: "brave".to_string(),
            name: "Brave Search".to_string(),
            search_url: "https://search.brave.com/search?q={searchTerms}".to_string(),
        })
}

pub fn get_default_dashboard_config(prefs_config: Option<&PreferencesInputConfig>) -> NewTabPage {
    prefs_config
        .map(|p| p.dashboard.clone())
        .unwrap_or_else(|| NewTabPage {
            show_clock: Some(true),
            show_background_image: Some(false),
            show_stats: Some(false),
            show_shortcuts: Some(false),
            show_branded_background_image: Some(false),
            show_cards: Some(false),
            show_search_widget: Some(false),
            show_brave_news: Some(false),
            show_together: Some(false),
        })
}

pub fn get_default_experimental_features(prefs_config: Option<&PreferencesInputConfig>) -> Vec<String> {
    prefs_config
        .map(|p| p.experimental_features.clone())
        .unwrap_or_else(|| vec!["brave-adblock-experimental-list-default@1".to_string()])
}