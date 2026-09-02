<h1 align="center">GitNapse Architecture</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#high-level">High-Level Design</a></li>
  <li><a href="#module-map">Module Map</a></li>
  <li><a href="#execution-flow">Execution Flow</a></li>
  <li><a href="#auth-strategy">Authentication Strategy</a></li>
  <li><a href="#config-strategy">Configuration Strategy</a></li>
  <li><a href="#provider-layer">Provider Layer</a></li>
  <li><a href="#theme-registry">Theme Registry</a></li>
  <li><a href="#cli">CLI Surface</a></li>
  <li><a href="#repository-boundary">Repository Boundary (gitnapse vs gitnapse/api)</a></li>
  <li><a href="#quality-gates">Quality Gates (local CI)</a></li>
</ul>

<h2 id="high-level" align="center">High-Level Design</h2>
<ul>
  <li><code>src/main.rs</code>: thin binary entrypoint that dispatches CLI actions into <code>dispatch_*</code> functions and boots the TUI.</li>
  <li><code>src/lib.rs</code>: public library surface of the core SDK (everything except the binary logic); re-exports <code>auth</code> submodules to keep <code>gitnapse::oauth</code>-style paths stable.</li>
  <li><code>src/app/</code>: TUI frontend — state machine, event loop, rendering and screens. Screens live in <code>src/app/screens/</code>.</li>
  <li><code>src/cli/</code>: headless commands (git operations, GitHub API actions, theme registry, auth) usable outside the TUI.</li>
  <li><code>src/github/</code>, <code>src/provider.rs</code>: GitHub HTTP client plus the <code>GitProvider</code> trait abstraction for future providers.</li>
  <li><code>src/auth/</code>: token resolution, keyring-backed secure storage, OAuth device flow and session persistence.</li>
  <li><code>src/config/</code>: account preferences, theme and keybinding configuration.</li>
  <li><code>src/models/</code>: DTO/domain models for GitHub responses and internal tree nodes.</li>
  <li><code>src/registry.rs</code>: client for the remote themes registry (<code>github.com/gitnapse/themes</code>).</li>
  <li><code>src/cache.rs</code>, <code>src/syntax.rs</code>, <code>src/task_manager.rs</code>, <code>src/runtime.rs</code>, <code>src/error.rs</code>: supporting infrastructure.</li>
</ul>

<h2 id="module-map" align="center">Module Map</h2>
<pre>
src/
  main.rs                 binary entrypoint: dispatch_* + TUI boot
  lib.rs                  public SDK exports
  app/                    TUI frontend (frontend, not SDK)
    mod.rs                state machine, event loop, key/mouse dispatch
    render.rs             layout and widget rendering
    theme.rs              palette strategy, navigation hints
    actions.rs            shared App action methods
    commands.rs           command palette actions
    network.rs            background network operations
    screens/              one module per screen
      search.rs           repository search screen
      tree.rs             repository tree + preview screen
      command_palette.rs  fuzzy command palette
    input/                nav.rs, mouse.rs, fields.rs
  cli/                    headless commands
    args.rs               clap definitions
    api.rs                GitHub API subcommands
    git.rs                git operations subcommands
    helpers.rs            shared CLI helpers
  github/                 GitHub client split by concern
    mod.rs                client, retries, headers, tests
    provider_impl.rs      GitProvider trait impl
    content.rs, prs.rs, compare.rs, repos.rs, ci.rs, releases.rs
  provider.rs             GitProvider trait, factory, URL detection
  auth/                   mod.rs, oauth.rs, oauth_session.rs, secure_store.rs
  config/                 mod.rs, account.rs, theme.rs, keybindings.rs
  models/                 mod.rs, repo.rs, pr.rs, release.rs, misc.rs
  registry.rs             remote themes registry client
  cache.rs, syntax.rs, task_manager.rs, runtime.rs, error.rs
</pre>

<h2 id="execution-flow" align="center">Execution Flow</h2>
<ol>
  <li>App resolves token from environment or secure storage (keyring with file fallback).</li>
  <li>Provider is created through the factory (<code>create_provider</code>) and initialized with optional bearer auth.</li>
  <li>TUI starts with default query and paginated repository search.</li>
  <li>User opens a repository and selects a branch when needed.</li>
  <li>Tree is loaded and lazily revealed for large repositories.</li>
  <li>File preview is loaded from cache or API and can be scrolled/focused.</li>
  <li>User can clone the repo, download the previewed file, or run any command-palette action.</li>
</ol>

<h2 id="auth-strategy" align="center">Authentication Strategy</h2>
<ul>
  <li>Preferred source: <code>GITHUB_TOKEN</code> environment variable.</li>
  <li>Fallback source: OS keyring via <code>keyring</code>, with a <code>0600</code> file fallback when no keyring is available (<code>src/auth/secure_store.rs</code>).</li>
  <li>OAuth device flow available via <code>gitnapse auth oauth login</code> using octocrab (<code>src/auth/oauth.rs</code>).</li>
  <li>OAuth session metadata persisted to support token lifecycle handling and optional refresh (<code>src/auth/oauth_session.rs</code>), zeroized on drop.</li>
  <li>Token input inside the TUI is validated against <code>/user</code>; an invalid stored token is cleared automatically.</li>
  <li>Client secrets are only read from environment variables (<code>GITNAPSE_GITHUB_OAUTH_CLIENT_SECRET</code>/<code>GITHUB_CLIENT_SECRET</code>); secrets never enter the config files.</li>
</ul>

<h2 id="config-strategy" align="center">Configuration Strategy</h2>
<ul>
  <li>Persisted under the user config directory:
    <ul>
      <li><code>account.json</code> — account preferences (<code>preferred_clone_dir</code>, <code>last_branch_by_repo</code>, …).</li>
      <li><code>theme.jsonc</code> — theme selection and color overrides.</li>
      <li><code>keybindings.jsonc</code> — customizable key bindings.</li>
      <li><code>themes/</code> — user-installed theme files (via <code>gitnapse theme install</code>).</li>
    </ul>
  </li>
  <li>Keyring values are never persisted inside <code>account.json</code>.</li>
</ul>

<h2 id="provider-layer" align="center">Provider Layer</h2>
<ul>
  <li><code>GitProvider</code> trait (~28 methods) in <code>src/provider.rs</code> abstracts all API operations; <code>src/github/provider_impl.rs</code> is the GitHub implementation backed by octocrab.</li>
  <li><code>detect_provider(url)</code> parses git remote URLs to identify the provider.</li>
  <li><code>create_provider(kind, token)</code> returns <code>Arc&lt;dyn GitProvider&gt;</code>; the TUI, CLI and (in the future) the HTTP server all consume this single abstraction.</li>
  <li>All network calls share the unified tokio runtime in <code>src/runtime.rs</code> and honor a 30s request timeout with retry/backoff.</li>
</ul>

<h2 id="theme-registry" align="center">Theme Registry</h2>
<ul>
  <li>Themes come from the remote registry <code>github.com/gitnapse/themes</code> (public data repo).</li>
  <li><code>index.json</code> at the registry root lists themes; each theme file lives under <code>colors/</code> and is fetched from <code>raw.githubusercontent.com</code>.</li>
  <li><code>gitnapse theme list --remote</code>, <code>gitnapse theme install &lt;name&gt;</code> and <code>gitnapse theme uninstall &lt;name&gt;</code> manage them.</li>
  <li>The index is untrusted input: theme names and <code>file</code> values are validated before becoming local file names or URLs.</li>
  <li>12 presets remain embedded in the binary via <code>include_str!</code> so <code>cargo install</code> keeps working offline.</li>
</ul>

<h2 id="cli" align="center">CLI Surface</h2>
<ul>
  <li><code>gitnapse clone &lt;owner/repo&gt;[:branch]</code>, issue/stash/tag management and other git operations run through <code>src/cli/git.rs</code>.</li>
  <li>GitHub API actions (search, PRs, issues, CI checks, comparisons, releases, repo management) run through <code>src/cli/api.rs</code>.</li>
  <li><code>src/cli/helpers.rs</code> centralizes error formatting and shared setup (git presence checks, provider creation).</li>
</ul>

<h2 id="repository-boundary" align="center">Repository Boundary (gitnapse vs gitnapse/api)</h2>
<ul>
  <li><strong>gitnapse</strong> is the application core: TUI frontend plus the SDK modules listed above. It owns the provider layer, auth, models, cache and config.</li>
  <li><strong>gitnapse/api</strong> holds the HTTP/JSON communication layer for web interfaces and future third-party integrations:
    <ul>
      <li><code>gitnapse-protocol</code> — the stable wire contract (DTOs), language/transport agnostic and independent from the core.</li>
      <li><code>gitnapse-server</code> — reference axum implementation that consumes the <code>gitnapse</code> SDK in-process and embeds a minimal web UI.</li>
    </ul>
  </li>
  <li>Rule: shared logic must be consumed by dependency (crate), never copied. The GitHub client lives once, in the core.</li>
</ul>

<h2 id="quality-gates" align="center">Quality Gates (local CI)</h2>
<ul>
  <li>CI is intentionally local-only; there are no remote GitHub Actions quality gates.</li>
  <li><code>scripts/ci.sh</code> is the single gate before merging: <code>cargo fmt</code> check, <code>cargo clippy -D warnings</code>, full test suite and <code>cargo audit</code>.</li>
</ul>
