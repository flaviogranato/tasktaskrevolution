use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct TemplateAssets;

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct StaticAssets;
