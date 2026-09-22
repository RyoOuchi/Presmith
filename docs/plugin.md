# Codex plugin and standalone skill

The repository contains an instruction-only plugin at `plugins/presmith/` with
`skills/presmith/SKILL.md` and self-contained workflow references. It does not include
hooks, apps, MCP, model API credentials or a CLI binary. Install/build the Rust CLI
separately and make `presmith` available in the coding agent's PATH.

## Local plugin installation (opt-in)

Current official packaging guidance supports a portable root `plugin.json` and a
`.codex-plugin/plugin.json` compatibility overlay. Both are included. Skill discovery
uses `skills/`. The plugin is published by RyoOuchi under the MIT license.
A downloadable plugin archive is included in the
[Presmith release](https://github.com/RyoOuchi/Presmith/releases/tag/v0.2.0).

From this repository, **if you want to make a local marketplace available**:

```sh
mkdir -p .agents/plugins
# Only copy this when there is no existing marketplace.json to preserve.
cp docs/plugin-marketplace.example.json .agents/plugins/marketplace.json
codex plugin marketplace add "$PWD"
codex plugin add presmith@presmith-local
```

If a repository catalog already exists, merge the sample's one plugin entry into
it and use that catalog's name in the add command. Do not replace other entries.
The source path is relative to the repository/marketplace root, not to .agents/plugins.
You can instead open the local marketplace in the desktop Plugins Directory and
install there. Start a new task after installation; if discovery is stale, restart
the desktop app. Reinstall after updating plugin contents; update the version when
needed to invalidate a cached copy.

**Verification boundary:** command syntax was checked against the installed Codex
CLI's `plugin add --help` and `plugin marketplace add --help`, and the instructions
were checked against official documentation. No marketplace was registered and no
plugin was installed into the user's account/configuration. Desktop discovery and
activation are therefore not claimed as tested.

## Standalone, project-local skill (opt-in)

From the target deck's directory (replace the source path with this checkout):

```sh
mkdir -p .agents/skills
cp -R /absolute/path/to/Presmith/plugins/presmith/skills/presmith .agents/skills/presmith
```

Do not overwrite an existing skill directory; update its contents deliberately.
Codex supports `.agents/skills/` discovery and symlinked skill folders. Copying keeps
all references inside the installed skill. Invoke `$presmith` in a task opened in
the deck project. No personal marketplace or global configuration change is needed.
This standalone copy/discovery flow is documented, not installed automatically.

## Validation evidence

The skill passes the available `skill-creator/scripts/quick_validate.py`. The root
plugin manifest validates against the official Agent Plugins 1.0.0 JSON schema.
The `.codex-plugin/plugin.json` compatibility manifest includes `author` and
`interface.developerName`, using the repository owner RyoOuchi. The release also
validates this manifest with `plugin-creator/scripts/validate_plugin.py`.

Sources checked during implementation:

- [Official plugin packaging and local marketplaces](https://developers.openai.com/plugins/build/plugins)
- [Official skill format and local discovery](https://learn.chatgpt.com/docs/build-skills)
- [Agent Plugins 1.0.0 schema](https://agent-plugins.org/schemas/1.0.0/plugin.schema.json)
