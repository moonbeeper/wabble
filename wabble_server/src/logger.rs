use tracing_subscriber::{
    EnvFilter, Layer as _, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

use crate::settings::{LoggingSettings, LoggingSettingsFormat};

pub fn init(settings: &LoggingSettings) {
    if !settings.enabled {
        return;
    }

    let layer = tracing_subscriber::fmt::layer()
        .with_line_number(settings.show_line_numbers)
        .with_thread_ids(settings.show_thread_ids)
        .with_file(settings.show_file_info);

    let layer = match settings.format {
        LoggingSettingsFormat::Normal => layer.boxed(),
        LoggingSettingsFormat::Pretty => layer.pretty().boxed(),
        LoggingSettingsFormat::Compact => layer.compact().boxed(),
    };

    tracing_subscriber::registry()
        .with(layer.with_filter(EnvFilter::from(&settings.level)))
        .init();
}
