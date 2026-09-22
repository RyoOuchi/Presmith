# Codex plugin and standalone skill

The repository contains an instruction-only plugin at `plugins/presmith/` with
`skills/presmith/SKILL.md` and self-contained workflow references. It does not include
hooks, apps, MCP, model API credentials or a CLI binary. Install/build the Rust CLI
separately and make `presmith` available in the coding agent's PATH.

## Bundled skill

Version 0.2.1 and newer embed the complete `skills/presmith/` folder in the Presmith
executable, including references and `agents/openai.yaml`. Every new
`presmith init DIRECTORY` project contains it at `.agents/skills/presmith/`.
Codex discovers that project-local folder when working in the deck. Invoke
`$presmith`; restart Codex if discovery has not refreshed. This needs no separate
plugin installation, Codex CLI, renderer setup or network access.

For a user-wide copy, run:

```sh
presmith skill install --global
presmith skill install --global --json
```

The destination is `$HOME/.agents/skills/presmith` on Unix, or
`%USERPROFILE%/.agents/skills/presmith` on Windows. Only this standalone skill is
installed; Codex settings, plugins and marketplace registrations are unchanged.
An identical copy is a successful no-op. To replace a different copy:

```sh
presmith skill install --global --force
```

The entire previous folder, including custom files, is saved under
`~/.agents/.presmith-skill-backups/install-*/presmith/` before replacement. Backups
are outside `.agents/skills/` so they do not appear as duplicate skills. A failed
replacement attempts to restore the previous directory; if restoration fails,
the error identifies the saved copy to restore manually. Symlink destinations and
symlinked `.agents` or `skills` directories are refused, even with `--force`.
Concurrent installers cannot replace the same skill simultaneously.

Existing project-local copies are unchanged when the CLI or global skill updates.
Codex can list project, user and plugin copies with the same name separately; it
does not merge them. Choose the installation scope you need. The v0.2.0 binary does not contain these
commands; upgrade to v0.2.1 or newer, or use the manual copy instructions below.

## Local plugin installation (opt-in)

Current official packaging guidance supports a portable root `plugin.json` and a
`.codex-plugin/plugin.json` compatibility overlay. Both are included. Skill discovery
uses `skills/`. The plugin is published by RyoOuchi under the MIT license.
A downloadable plugin archive is included in the
[Presmith release](https://github.com/RyoOuchi/Presmith/releases/tag/v0.2.2).

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

### Finding Presmith in Codex

The plugin's display name is **Presmith**, and the skill is invoked as `$presmith`.
The skill includes `agents/openai.yaml` so the skill picker also displays Presmith.
Start a new task after installation; restart Codex if the picker is still stale.

An installation named **Decksmith** is an older package. Renaming this checkout or
editing its skill does not update the separately installed plugin. Register the
Presmith package in the marketplace you use, install `presmith@MARKETPLACE`, and
then remove the old `decksmith@MARKETPLACE` installation. For a personal marketplace,
use the plugin-creator scaffold and update helpers to register and refresh the
local package. Check `codex plugin list --marketplace MARKETPLACE --json` to confirm
the installed name before trying `$presmith` in a new task.

**Verification boundary:** command syntax was checked against the installed Codex
CLI's help and official documentation. Local installation and enabled status can
be verified with `codex plugin list --marketplace MARKETPLACE --json`. Check the
desktop skill picker in a new task separately; CLI installation alone does not
verify that an already-open task has refreshed its skills.

## Existing decks and older binaries

New decks already contain the skill. To add it to an older deck without
reinitializing that deck, copy from the checkout (replace the source path):

```sh
mkdir -p .agents/skills
cp -R /absolute/path/to/Presmith/plugins/presmith/skills/presmith .agents/skills/presmith
```

Do not overwrite an existing skill directory; update its contents deliberately.
Codex supports `.agents/skills/` discovery and symlinked skill folders. Copying keeps
all references inside the installed skill. Invoke `$presmith` in a task opened in
the deck project. No personal marketplace or global configuration change is needed.
The copy keeps the skill scoped to this project. Global installation above is
an alternative when you want the same skill available across existing projects.

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
