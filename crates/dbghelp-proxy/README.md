# dbghelp-proxy

This crate provides a proxy for the `dbghelp.dll` library.
It is the entry point for the mod loader patcher and runtime.

## Running with Wine/Proton on Linux

Using `dbghelp-proxy` with Wine/Proton requires some additional setup.

For clients on Steam, change the launch options (Enshrouded > Properties > General > Launch Options) to:

```bash
WINEDLLOVERRIDES="dbghelp=native,builtin" %command%
```

For servers or standalone execution, run the game executable with:

```bash
WINEDLLOVERRIDES="dbghelp=native,builtin" wine path/to/enshrouded.exe
```

or with Proton:

```bash
WINEDLLOVERRIDES="dbghelp=native,builtin" proton run path/to/enshrouded.exe
```

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
