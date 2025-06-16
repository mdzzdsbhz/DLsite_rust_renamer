use eframe::egui::{FontDefinitions, FontFamily, Context};

pub fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    // 加载系统字体或者自定义路径字体（这里用的是微软雅黑作为示例）
    fonts.font_data.insert(
        "my_chinese_font".to_owned(),
        std::sync::Arc::new(eframe::egui::FontData::from_owned(
            include_bytes!("../fonts/msyh.ttf").to_vec(), // 字体路径
        )),
    );

    // 设置用于 Proportional 和 Monospace 的字体族优先级
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "my_chinese_font".to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "my_chinese_font".to_owned());

    ctx.set_fonts(fonts);
}