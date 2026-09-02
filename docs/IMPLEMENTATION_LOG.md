<h1 align="center">GitNapse Implementation Log</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#scope">Scope Materialization</a></li>
  <li><a href="#foundation">Completed Foundation</a></li>
  <li><a href="#features">Completed Functional Features</a></li>
  <li><a href="#quality">Quality and Validation</a></li>
  <li><a href="#opportunities">Known Iteration Opportunities</a></li>
</ul>

<h2 id="scope" align="center">Scope Materialization</h2>
<p>This log tracks the implemented baseline and subsequent functional expansions.</p>

<h2 id="foundation" align="center">Completed Foundation</h2>
<ul>
  <li>Rust binary project initialized and organized in modular architecture.</li>
  <li>Dependencies updated to modern versions (including TUI and API layers).</li>
  <li>Core modules established for:
    <ul>
      <li>application state machine</li>
      <li>rendering and theme</li>
      <li>authentication and account settings</li>
      <li>GitHub API integration</li>
      <li>preview syntax and cache</li>
    </ul>
  </li>
</ul>

<h2 id="features" align="center">Completed Functional Features</h2>
<ul>
  <li>Paginated repository search with keyboard page controls.</li>
  <li>Branch-aware repository exploration and recursive tree retrieval.</li>
  <li>Lazy tree reveal for large repositories.</li>
  <li>Preview pane with dedicated focus, viewport range, and fast scrolling controls.</li>
  <li>Mouse integration:
    <ul>
      <li>single click selection</li>
      <li>double click open for repos/files</li>
      <li>wheel scroll in tree and preview panes</li>
    </ul>
  </li>
  <li>Escape back-navigation from repo view to search list.</li>
  <li>In-app clone modal with editable destination path.</li>
  <li>In-app download modal for current previewed file.</li>
  <li>CLI single-file download command (<code>gitnapse download-file</code>).</li>
  <li>Tree file-name search shortcut and full tree-text view toggle.</li>
  <li>Token management commands and runtime validation against GitHub user endpoint.</li>
  <li>OAuth device-flow login implemented with <code>octocrab</code> and secure token persistence.</li>
  <li>OAuth session lifecycle handling added (expiry metadata + optional refresh flow with client secret env variables).</li>
  <li>Full palette-based navigation coloring with contrast-safe foreground.</li>
  <li>Headless CLI expanded: local git operations and GitHub API actions via <code>src/cli/</code> (git.rs, api.rs, helpers.rs).</li>
  <li>Provider abstraction (<code>GitProvider</code> trait) with GitHub implementation, URL auto-detection and factory.</li>
  <li>Modularized screens under <code>src/app/screens/</code> (search, tree, command palette) and <code>auth/</code> submodules.</li>
  <li>Remote theme registry (<code>github.com/gitnapse/themes</code>) with <code>theme list/install/uninstall</code> CLI and validated inputs.</li>
  <li>API communication layer extracted to the <code>gitnapse/api</code> repository (<code>gitnapse-protocol</code> + <code>gitnapse-server</code>) for web interfaces and future third-party integrations.</li>
</ul>

<h2 id="quality" align="center">Quality and Validation</h2>
<ul>
  <li><code>scripts/ci.sh</code> passes: <code>cargo fmt --check</code>, <code>cargo clippy --all-targets --all-features -- -D warnings</code>, full test suite, and <code>cargo audit</code>.</li>
  <li>CI is run locally; no remote GitHub Actions quality gates.</li>
</ul>

<h2 id="opportunities" align="center">Known Iteration Opportunities</h2>
<ul>
  <li>Richer language-aware syntax highlighting engine.</li>
  <li>Additional integration tests for GitHub client and clone/download workflows.</li>
  <li>Optional advanced filtering/sorting in repository and tree navigation views.</li>
  <li>SDK/library boundary: feature-gate the TUI so <code>gitnapse/api</code> consumes only the core modules.</li>
</ul>
