use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchProvider {
    pub keyword: String,
    pub name: String,
    pub search_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTabPage {
    pub show_clock: Option<bool>,
    pub show_background_image: Option<bool>,
    pub show_stats: Option<bool>,
    pub show_shortcuts: Option<bool>,
    pub show_branded_background_image: Option<bool>,
    pub show_cards: Option<bool>,
    pub show_search_widget: Option<bool>,
    pub show_brave_news: Option<bool>,
    pub show_together: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BraveStats {
    pub enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BraveToday {
    pub should_show_brave_today_widget: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BravePreferences {
    pub new_tab_page: Option<NewTabPage>,
    pub stats: Option<BraveStats>,
    pub today: Option<BraveToday>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BrowserPreferences {
    pub enabled_labs_experiments: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserPreferences {
    pub default_search_provider_data: Option<SearchProvider>,
    pub brave: Option<BravePreferences>,
    pub browser: Option<BrowserPreferences>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalState {
    pub browser: Option<BrowserPreferences>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreferencesInputConfig {
    pub search_engines: Vec<SearchProvider>,
    pub dashboard: NewTabPage,
    pub experimental_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreferencesConfig {
    pub preferences: Option<UserPreferences>,
    pub local_state: Option<LocalState>,
}