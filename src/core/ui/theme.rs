use super::types::UiThemeConfig;

pub fn apply_theme(ctx: &egui::Context, theme: &UiThemeConfig) {
    let mut fonts = egui::FontDefinitions::default();
    for font in &theme.fonts {
        fonts
            .font_data
            .insert(font.name.clone(), egui::FontData::from_owned(font.data.clone()));
        let family_name = font.family.clone().unwrap_or_else(|| "proportional".into());
        let family = parse_font_family(&family_name);
        fonts
            .families
            .entry(family)
            .or_default()
            .push(font.name.clone());
    }

    for (family_name, list) in &theme.font_families {
        let family = parse_font_family(family_name);
        fonts.families.insert(family, list.clone());
    }

    ctx.set_fonts(fonts);

    if !theme.text_styles.is_empty() {
        let mut style = (*ctx.style()).clone();
        for (name, text_style) in &theme.text_styles {
            let family = text_style
                .family
                .as_deref()
                .map(parse_font_family)
                .unwrap_or(egui::FontFamily::Proportional);
            let text_style_id = egui::TextStyle::Name(name.clone().into());
            style
                .text_styles
                .insert(text_style_id, egui::FontId::new(text_style.size, family));
        }
        ctx.set_style(style);
    }
}

fn parse_font_family(name: &str) -> egui::FontFamily {
    match name {
        "proportional" => egui::FontFamily::Proportional,
        "monospace" => egui::FontFamily::Monospace,
        _ => egui::FontFamily::Name(name.into()),
    }
}
