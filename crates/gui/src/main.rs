//! DeepSeek GUI — native desktop frontend built with egui/eframe.

use anyhow::Result;
use deepseek_gui::app::GuiApp;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DeepSeek",
        options,
        Box::new(|cc| Ok(Box::new(GuiApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))?;

    Ok(())
}
