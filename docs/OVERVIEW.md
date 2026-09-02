<h1 align="center">GitNapse Overview</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#what-is-gitnapse">What is GitNapse</a></li>
  <li><a href="#features">Features</a></li>
  <li><a href="#authentication">Authentication</a></li>
  <li><a href="#theming">Theming</a></li>
  <li><a href="#configuration">Configuration</a></li>
  <li><a href="#architecture">Architecture</a></li>
</ul>

<h2 id="what-is-gitnapse" align="center">What is GitNapse</h2>
<p>
  GitNapse is a Terminal User Interface (TUI) application for browsing, exploring, and managing
  GitHub repositories directly from your terminal. It is written in Rust and uses
  <code>ratatui</code> for rendering and <code>crossterm</code> for terminal interaction.
</p>

<h2 id="features" align="center">Features</h2>

<h3 align="center">Repository Exploration</h3>
<ul>
  <li>Search GitHub repositories by query string or username</li>
  <li>Authenticated <code>@me</code> query mode to list your own repos (including private)</li>
  <li>Repository pagination (next/previous page)</li>
  <li>Open any repo to browse its file tree with lazy loading for large trees</li>
  <li>File preview with syntax highlighting (Rust, Python, JavaScript, Go, C, and more)</li>
  <li>Branch picker to switch between branches</li>
  <li>Fuzzy file finder (character-order matching) to locate files in the tree</li>
  <li>Tree text view to see the full repository tree in the preview pane</li>
  <li>Clone the repository to a local path</li>
  <li>Download individual previewed files</li>
  <li>Multi-select repositories with the space bar</li>
</ul>

<h3 align="center">Command Palette</h3>
<p>
  Press <code>Ctrl+P</code> to open a VS Code-style command palette with fuzzy search over
  all available actions. Type to filter, arrows to navigate, Enter to execute, Esc to close.
</p>
<ul>
  <li>Search Repositories</li>
  <li>List Starred Repositories</li>
  <li>Switch Branch, Find File, Clone Repository, Download Current File</li>
  <li>Change Theme (browse and switch between 12 built-in themes)</li>
  <li>View PR Detail, Create Pull Request</li>
  <li>List Issues, List Pull Requests</li>
  <li>View Recent Commits, View CI Status, Compare Branches</li>
  <li>Set Token</li>
  <li>Quit</li>
</ul>

<h3 align="center">Pull Request Management</h3>
<ul>
  <li>View PR detail: title, body, status, file stats, branches, labels</li>
  <li>Submit reviews: Approve, Request Changes, or Comment with custom text</li>
  <li>Merge PRs with three methods: merge commit, squash, or rebase</li>
  <li>Close PRs</li>
  <li>Browse reviews, inline comments, and commits for any PR</li>
  <li>Create PRs via a 4-step guided wizard (title, head branch, base branch, description)</li>
  <li>All operations run on background threads -- the UI stays responsive</li>
</ul>

<h3 align="center">CI and Repository Insights</h3>
<ul>
  <li>View GitHub Actions check runs for the active branch</li>
  <li>View workflow runs</li>
  <li>Compare two branches (ahead/behind, file diff stats)</li>
  <li>View recent commits for the active branch</li>
  <li>List open issues and pull requests</li>
</ul>

<h2 id="authentication" align="center">Authentication</h2>
<p>
  GitNapse supports multiple authentication sources with a clear precedence order:
</p>
<ol>
  <li><strong>Environment variable</strong> <code>GITHUB_TOKEN</code></li>
  <li><strong>OAuth session</strong> with automatic token refresh when expired</li>
  <li><strong>Keyring</strong> via the operating system's native credential store</li>
  <li><strong>File fallback</strong> in the config directory with restricted permissions (0600)</li>
</ol>
<p>
  OAuth login uses GitHub's device flow via <code>octocrab</code>. It opens your browser,
  displays a device code, and stores the resulting access token securely. Refresh tokens
  are supported for extended sessions.
</p>

<h2 id="theming" align="center">Theming</h2>
<p>
  GitNapse ships with 12 built-in color themes: X, Madrid, Lahabana, Miami, Paris, Tokio,
  Oslo, Helsinki, Berlin, London, Praha, and Bogota. Themes define a 16-color palette used
  for selection highlighting. Each theme is verified for WCAG contrast compliance.
</p>
<p>
  Switch themes from the command palette. Your selection is persisted and restored on next
  launch. The 12 presets are embedded in the binary, so they work offline. Custom themes can
  be added as <code>.jsonc</code> files in the config directory's <code>themes/</code> folder,
  or installed from the remote registry (<code>github.com/gitnapse/themes</code>) with
  <code>gitnapse theme install &lt;name&gt;</code>. See <a href="THEME_CONFIG.md">THEME_CONFIG.md</a> for the format.
</p>

<h2 id="configuration" align="center">Configuration</h2>
<p>GitNapse stores configuration in a platform-appropriate config directory:</p>
<ul>
  <li>Linux: <code>~/.config/GitNapse/</code></li>
  <li>macOS: <code>~/Library/Application Support/com.GitNapse.GitNapse/</code></li>
  <li>Windows: <code>C:\Users\&lt;user&gt;\AppData\Roaming\GitNapse\GitNapse\config\</code></li>
</ul>

<table>
  <thead>
    <tr><th>File</th><th>Purpose</th></tr>
  </thead>
  <tbody>
    <tr><td><code>account.json</code></td><td>Preferred clone directory, last branch per repo, last selected theme</td></tr>
    <tr><td><code>theme.jsonc</code></td><td>Custom color palette configuration (16 RGB colors)</td></tr>
    <tr><td><code>keybindings.jsonc</code></td><td>Custom keybinding overrides (optional, defaults used if absent)</td></tr>
    <tr><td><code>themes/*.jsonc</code></td><td>User themes (custom or installed from the remote registry)</td></tr>
    <tr><td><code>token</code></td><td>Stored GitHub token (encrypted via keyring, with file fallback)</td></tr>
    <tr><td><code>oauth_session.json</code></td><td>OAuth session metadata including refresh tokens</td></tr>
  </tbody>
</table>

<h2 id="architecture" align="center">Architecture</h2>
<p>
  The codebase is organized into modular directories:
</p>
<ul>
  <li><code>src/main.rs</code> -- thin binary entrypoint dispatching CLI actions</li>
  <li><code>src/app/</code> -- TUI frontend (state, input handling, rendering, commands, network event processing); screens under <code>src/app/screens/</code></li>
  <li><code>src/cli/</code> -- headless commands (git operations, GitHub API, themes registry, auth)</li>
  <li><code>src/github/</code> -- GitHub REST API client with typed error handling and retry logic</li>
  <li><code>src/provider.rs</code> -- <code>GitProvider</code> trait abstraction for future providers</li>
  <li><code>src/config/</code> -- Configuration management (account, themes, keybindings)</li>
  <li><code>src/models/</code> -- Data models for all GitHub API responses</li>
  <li><code>src/auth/</code> -- Authentication (token loading, OAuth device flow, session, secure storage)</li>
  <li><code>src/registry.rs</code> -- Client for the remote themes registry</li>
  <li><code>src/cache.rs</code> -- Preview cache with TTL and ETag support</li>
  <li><code>src/syntax.rs</code> -- Syntax highlighting engine</li>
  <li><code>src/error.rs</code> -- Typed error enums via <code>thiserror</code></li>
  <li><code>src/task_manager.rs</code>, <code>src/runtime.rs</code> -- background threads and shared tokio runtime</li>
</ul>
<p>
  Network operations run on background threads via <code>mpsc</code> channels, keeping the
  TUI responsive during API calls. The GitHub client uses <code>reqwest</code> (async) with
  a shared tokio runtime and automatic retry on transient errors.
</p>
