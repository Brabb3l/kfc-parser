# dinput8-proxy

This crate provides a proxy for the `dinput8.dll` library.
It is the entry point for the mod loader patcher and runtime.

## Development on Steam (Linux Only)

If you want to have a console for debugging purposes,
change the Launch Options of the game in Steam to:

```bash
EML_CONSOLE=1 %command%
```

## Environment Variables

- `EML_CONSOLE`: Enable or disable the console. Default is `false`.
- `EML_LOG_FILE_FILTER`: Filter for file logging. [format](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives)
- `EML_LOG_FILE_ENABLED`: Enable or disable file logging. Default is `true`.
- `EML_LOG_FILE_PATH`: Path to the log file. Default is `./logs`
- `EML_LOG_FILE_MAX`: Maximum amount of log files to keep. Default is `128`.
- `EML_LOG_STDOUT_FILTER`: Filter for stdout logging. [format](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives)
- `EML_LOG_STDOUT_ENABLED`: Enable or disable stdout logging. Default is `true
