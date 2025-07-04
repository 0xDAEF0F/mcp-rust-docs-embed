use colored::Colorize;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
	fmt::{self, FmtContext, FormatEvent, FormatFields},
	registry::LookupSpan,
};

pub struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
	S: Subscriber + for<'a> LookupSpan<'a>,
	N: for<'a> FormatFields<'a> + 'static,
{
	fn format_event(
		&self,
		ctx: &FmtContext<'_, S, N>,
		mut writer: fmt::format::Writer<'_>,
		event: &Event<'_>,
	) -> std::fmt::Result {
		let meta = event.metadata();
		let level = meta.level();

		// format level with color
		let level_str = match *level {
			Level::ERROR => "ERROR".red(),
			Level::WARN => "WARN".yellow(),
			Level::INFO => "INFO".green(),
			Level::DEBUG => "DEBUG".blue(),
			Level::TRACE => "TRACE".purple(),
		};
		write!(writer, "[{level_str}] ")?;

		// format target
		write!(writer, "[{}]: ", meta.target())?;

		// format fields
		ctx.field_format().format_fields(writer.by_ref(), event)?;

		writeln!(writer)
	}
}
