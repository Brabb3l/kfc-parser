# Using EML or ShroudForge

This update preserves the existing EML setup and adds support for ShroudForge.
EML-only installations continue to start EML as before, with no configuration
change required.

On installations with ShroudForge, the `modloader` setting in
`shroudforge/config/modloader-config.json` selects which loader the proxy starts:

- `"modloader": "eml"` keeps EML active.
- `"modloader": "shroudforge"` starts ShroudForge. This is the default in
  ShroudForge's configuration.

The proxy starts only the selected loader. When ShroudForge is selected, the
proxy hands startup to ShroudForge's `winmm.dll` bootstrap. If the bootstrap
cannot be loaded, game startup stops and the proxy attempts to record the error
in ShroudForge's pending log. To run EML instead, select `"eml"` in the setting.
