<h1 align="center">GitNapse Usage Guide</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#requirements">Requirements</a></li>
  <li><a href="#cli-table">CLI Command Table</a></li>
  <li><a href="#in-app-controls">In-App Control Table</a></li>
  <li><a href="#command-palette">Command Palette</a></li>
  <li><a href="#pr-management">Pull Request Management</a></li>
  <li><a href="#multi-select">Multi-Select Repositories</a></li>
  <li><a href="#fuzzy-file-search">Fuzzy File Search</a></li>
  <li><a href="#themes">Theme System</a></li>
  <li><a href="#keybindings-config">Keybindings Configuration</a></li>
  <li><a href="#my-private-repos">My Private Repositories</a></li>
  <li><a href="#workflows">Core Workflows</a></li>
  <li><a href="#troubleshooting">Troubleshooting</a></li>
</ul>

<h2 id="requirements" align="center">Requirements</h2>
<ul>
  <li>Rust toolchain: <code>cargo</code>, <code>rustc</code></li>
  <li>Internet connection for GitHub API requests</li>
  <li>Local <code>git</code> available in <code>PATH</code></li>
</ul>

<h2 id="cli-table" align="center">CLI Command Reference</h2>
<p>
  The full CLI command reference has moved to <a href="./GITNAPSE_CLI.md"><code>GITNAPSE_CLI.md</code></a>.
  It covers all commands: <code>clone</code>, <code>commit</code>, <code>push</code>, <code>pull</code>,
  <code>fetch</code>, <code>checkout</code>, <code>diff</code>, <code>stash</code>, <code>tag</code>,
  <code>status</code>, <code>log</code>, <code>branch</code>, <code>reset</code>, <code>remote</code>,
  <code>config</code>, <code>merge</code>, <code>pr</code>, <code>issue</code>, <code>ci</code>,
  <code>compare</code>, <code>search</code>, <code>release</code>, and <code>repo</code>.
</p>

<h2 id="in-app-controls" align="center">In-App Control Table</h2>
<table>
  <thead>
    <tr>
      <th>Key / Input</th>
      <th>Area</th>
      <th>Action</th>
      <th>Details</th>
    </tr>
  </thead>
  <tbody>
    <tr><td><code>Ctrl+P</code></td><td>Global</td><td>Command palette</td><td>Open the command palette with fuzzy search over all available actions</td></tr>
    <tr><td><code>/</code></td><td>Global</td><td>Open search input</td><td>Edit repository search query</td></tr>
    <tr><td><code>Enter</code></td><td>Contextual</td><td>Execute / open / preview</td><td>Search, open repo, or preview selected file</td></tr>
    <tr><td><code>Tab</code></td><td>Global</td><td>Cycle focus</td><td><code>Repos -> Tree -> Preview</code></td></tr>
    <tr><td><code>Esc</code></td><td>Global</td><td>Back navigation</td><td>Close modal or return from repo view to result list</td></tr>
    <tr><td><code>Up / Down</code></td><td>Tree / Preview</td><td>Navigate / Scroll</td><td>Moves selection in tree or scrolls preview when focused</td></tr>
    <tr><td><code>PgUp / PgDn</code></td><td>Preview</td><td>Fast scroll</td><td>Page-sized preview movement</td></tr>
    <tr><td><code>Home / End</code></td><td>Preview</td><td>Jump bounds</td><td>Go to top / bottom of preview</td></tr>
    <tr><td><code>Left / [</code></td><td>Repos list</td><td>Previous page</td><td>Move to previous GitHub search page</td></tr>
    <tr><td><code>Right / ]</code></td><td>Repos list</td><td>Next page</td><td>Move to next GitHub search page</td></tr>
    <tr><td><code>Space</code></td><td>Repos list</td><td>Toggle multi-select</td><td>Select or deselect the current repository for batch operations</td></tr>
    <tr><td><code>b</code></td><td>Repo view</td><td>Branch picker</td><td>Open branch selector modal</td></tr>
    <tr><td><code>f</code></td><td>Repo view</td><td>File search (fuzzy)</td><td>Find file with fuzzy matching in loaded tree</td></tr>
    <tr><td><code>v</code></td><td>Repo view</td><td>Toggle tree text view</td><td>Show whole repository tree in preview pane</td></tr>
    <tr><td><code>c</code></td><td>Repo view</td><td>Clone modal</td><td>Prompt destination path and run clone</td></tr>
    <tr><td><code>d</code></td><td>Preview</td><td>Download modal</td><td>Save current previewed file to local path</td></tr>
    <tr><td><code>Del</code></td><td>Path modals</td><td>Clear path input</td><td>Works in clone/download path inputs</td></tr>
    <tr><td><code>t</code></td><td>Global</td><td>Token modal</td><td>Save token from inside the TUI</td></tr>
    <tr><td><code>o</code></td><td>Global</td><td>OAuth quick check</td><td>Does not start login; runs status check</td></tr>
    <tr><td><code>q</code></td><td>Global</td><td>Quit</td><td>Exit application</td></tr>
    <tr><td>Mouse left click</td><td>Tree / Preview / Repos</td><td>Focus and select</td><td>Single click selects, double click opens (repo/file)</td></tr>
    <tr><td>Mouse wheel</td><td>Tree / Preview</td><td>Scroll</td><td>Scroll behavior depends on pointer position</td></tr>
  </tbody>
</table>

<h2 id="command-palette" align="center">Command Palette</h2>
<p>
  Press <code>Ctrl+P</code> to open the command palette. This provides a VS Code-style searchable
  list of all available actions. Type to filter commands with substring matching, use
  <code>Up</code> / <code>Down</code> to navigate, <code>Enter</code> to execute, and
  <code>Esc</code> to close.
</p>
<table>
  <thead>
    <tr><th>Command</th><th>Context</th><th>Description</th></tr>
  </thead>
  <tbody>
    <tr><td>Search Repositories</td><td>Always</td><td>Focus the search input to query GitHub repos</td></tr>
    <tr><td>List Starred Repos</td><td>Always</td><td>Show your starred repositories from GitHub</td></tr>
    <tr><td>Change Theme</td><td>Always</td><td>Browse and switch between 12 built-in color themes</td></tr>
    <tr><td>Set Token</td><td>Always</td><td>Save a GitHub token for authenticated requests</td></tr>
    <tr><td>Switch Branch</td><td>Repo open</td><td>Open branch picker to switch the active branch</td></tr>
    <tr><td>Find File</td><td>Repo open</td><td>Fuzzy-search files in the repository tree</td></tr>
    <tr><td>Clone Repository</td><td>Repo open</td><td>Clone the current repo to a local path</td></tr>
    <tr><td>Download Current File</td><td>Preview active</td><td>Download the currently previewed file</td></tr>
    <tr><td>Toggle Tree View</td><td>Repo open</td><td>Show the full tree as text in the preview pane</td></tr>
    <tr><td>View PR Detail</td><td>Repo open</td><td>Enter a PR number and load its full detail</td></tr>
    <tr><td>Create Pull Request</td><td>Repo open</td><td>Multi-step wizard: title, head branch, base branch, description</td></tr>
    <tr><td>List Issues</td><td>Repo open</td><td>Display open issues for the current repo</td></tr>
    <tr><td>List Pull Requests</td><td>Repo open</td><td>Display open pull requests for the current repo</td></tr>
    <tr><td>View Recent Commits</td><td>Repo open</td><td>Show recent commits for the active branch</td></tr>
    <tr><td>View CI Status</td><td>Repo open</td><td>Display GitHub Actions check runs for the active branch</td></tr>
    <tr><td>Compare Branches</td><td>Repo open</td><td>Compare the current branch against another branch</td></tr>
    <tr><td>Quit</td><td>Always</td><td>Exit the application</td></tr>
  </tbody>
</table>

<h2 id="pr-management" align="center">Pull Request Management</h2>
<p>
  GitNapse supports viewing and managing pull requests directly from the TUI. To load a PR,
  use <code>Ctrl+P</code> and select <code>View PR Detail</code>, then enter the PR number
  when prompted.
</p>
<p>Once a PR is loaded, the following actions are available in the command palette:</p>
<table>
  <thead>
    <tr><th>Action</th><th>Description</th></tr>
  </thead>
  <tbody>
    <tr><td><code>[Approve]</code></td><td>Submit an approval review. Prompts for an optional comment (Enter submits, Esc cancels).</td></tr>
    <tr><td><code>[Request Changes]</code></td><td>Submit a change request review with an optional description.</td></tr>
    <tr><td><code>[Comment]</code></td><td>Post a general review comment without approval or change request.</td></tr>
    <tr><td><code>[Merge: merge commit]</code></td><td>Merge using a standard merge commit.</td></tr>
    <tr><td><code>[Merge: squash]</code></td><td>Squash all commits into a single commit and merge.</td></tr>
    <tr><td><code>[Merge: rebase]</code></td><td>Rebase commits onto the base branch and fast-forward merge.</td></tr>
    <tr><td><code>[Close PR]</code></td><td>Close the pull request without merging.</td></tr>
    <tr><td><code>[View Reviews]</code></td><td>Load all review submissions for the PR.</td></tr>
    <tr><td><code>[View Comments]</code></td><td>Load inline review comments on the PR diff.</td></tr>
    <tr><td><code>[View Commits]</code></td><td>Load the list of commits in the PR.</td></tr>
  </tbody>
</table>

<h3 id="create-pr" align="center">Creating a Pull Request</h3>
<p>
  Select <code>Create Pull Request</code> from the command palette. A guided 4-step wizard
  will prompt for:
</p>
<ol>
  <li><strong>Title</strong> -- the PR title (required)</li>
  <li><strong>Head branch</strong> -- the source branch containing your changes</li>
  <li><strong>Base branch</strong> -- the target branch (e.g. <code>main</code>)</li>
  <li><strong>Description</strong> -- optional body text for the PR</li>
</ol>
<p>
  The PR is created via the GitHub API and its detail is shown immediately.
</p>

<h2 id="multi-select" align="center">Multi-Select Repositories</h2>
<p>
  In the repository list, press <code>Space</code> to toggle selection of the current repo.
  Selected repos are marked with <code>[*]</code>. The active selection shows <code>&gt;[*]</code>
  and inactive selected ones show <code>[ ]</code>. There is no batch action implemented yet;
  this is infrastructure for future operations like bulk clone or bulk download.
</p>

<h2 id="fuzzy-file-search" align="center">Fuzzy File Search</h2>
<p>
  Press <code>f</code> in the repository tree view or select <code>Find File</code> from the
  command palette. The tree file search uses character-order fuzzy matching — all characters
  in your query must appear in order (not necessarily consecutively) in the target filename.
  Enter a search term and press <code>Enter</code> to jump to the first match.
  Press <code>Esc</code> to cancel.
</p>

<h2 id="themes" align="center">Theme System</h2>
<p>
  GitNapse ships with 12 built-in color themes: X, Madrid, Lahabana, Miami, Paris, Tokio,
  Oslo, Helsinki, Berlin, London, Praha, and Bogota. Themes define a 16-color palette
  used for selection highlighting across the UI.
</p>
<p>
  To change the theme, use <code>Ctrl+P</code> &gt; <code>Change Theme</code> and select
  from the list. Your choice is persisted to <code>account.json</code> and restored on
  next launch.
</p>
<p>
  Custom themes can be added by placing a <code>.jsonc</code> file in the config directory's
  <code>themes/</code> folder, or installed from the remote registry
  (<code>github.com/gitnapse/themes</code>) with
  <code>gitnapse theme install &lt;name&gt;</code> (see <code>gitnapse theme list --remote</code>
  for available names). See <a href="THEME_CONFIG.md">THEME_CONFIG.md</a> for the
  file format.
</p>

<h2 id="keybindings-config" align="center">Keybindings Configuration</h2>
<p>
  Keybindings can be customized by creating a <code>keybindings.jsonc</code> file in the
  GitNapse config directory. The file uses JSONC format (supports <code>//</code> comments).
  If the file does not exist, the built-in defaults are used.
</p>
<p>Example <code>keybindings.jsonc</code>:</p>
<pre><code>{
    // GitNapse Keybindings
    "quit": "q",
    "search": "/",
    "token_input": "t",
    "oauth_status": "o",
    "clone": "c",
    "branch_picker": "b",
    "file_search": "f",
    "download": "d",
    "tree_view": "v",
    "focus_next": "Tab",
    "back": "Esc",
    "page_left": ["Left", "["],
    "page_right": ["Right", "]"],
    "scroll_down": "Down",
    "scroll_up": "Up",
    "page_down": "PageDown",
    "page_up": "PageUp",
    "home": "Home",
    "end": "End",
    "enter": "Enter",
    "escape": "Esc",
    "backspace": "Backspace",
    "delete": "Delete"
}
</code></pre>

<h2 id="my-private-repos" align="center">My Private Repositories</h2>
<p>
  GitHub search endpoint does not guarantee full private-repository discovery by username query.
  To list your own repositories (including private ones), use the authenticated query mode:
</p>
<ul>
  <li>Inside TUI search input (<code>/</code>): <code>@me</code></li>
  <li>Optional text filter: <code>@me rust</code> or <code>me:rust</code></li>
  <li>Language filter: <code>@me language:rust</code> or <code>@me lang:javascript</code></li>
  <li>Combined filters: <code>@me language:rust private</code> or <code>@me language:rust,javascript api</code></li>
  <li>CLI start: <code>gitnapse run --query "@me"</code></li>
</ul>
<p>
  This mode requires a valid authenticated session/token and uses your account repository listing API scope.
</p>

<h2 id="workflows" align="center">Core Workflows</h2>

<h3 id="workflow-repo" align="center">Open and Explore a Repository</h3>
<ol>
  <li>Search repositories with <code>/</code> then <code>Enter</code>.</li>
  <li>Select one result and open it with <code>Enter</code> or double click.</li>
  <li>Browse files in tree pane; press <code>Enter</code> or double click file for preview.</li>
  <li>Use <code>Tab</code> to focus preview and scroll long content.</li>
</ol>

<h3 id="workflow-branch" align="center">Switch Branch</h3>
<ol>
  <li>Press <code>b</code> in repo view, or use <code>Ctrl+P</code> &gt; <code>Switch Branch</code>.</li>
  <li>Select branch with arrows.</li>
  <li>Press <code>Enter</code> to reload tree/preview context on that branch.</li>
</ol>

<h3 id="workflow-clone" align="center">Clone Repository</h3>
<ol>
  <li>Open repository view.</li>
  <li>Press <code>c</code> or use <code>Ctrl+P</code> &gt; <code>Clone Repository</code>.</li>
  <li>Set destination path and press <code>Enter</code>.</li>
</ol>

<h3 id="workflow-download" align="center">Download Current Previewed File</h3>
<ol>
  <li>Open preview for a file.</li>
  <li>Press <code>d</code> or use <code>Ctrl+P</code> &gt; <code>Download Current File</code>.</li>
  <li>Provide output path and press <code>Enter</code>.</li>
</ol>

<h3 id="workflow-pr-review" align="center">Review and Merge a Pull Request</h3>
<ol>
  <li>Open a repository, then use <code>Ctrl+P</code> &gt; <code>View PR Detail</code>.</li>
  <li>Enter the PR number when prompted and press <code>Enter</code>.</li>
  <li>The PR detail panel shows title, body, status, branches, and available actions.</li>
  <li>Select <code>[Approve]</code>, <code>[Request Changes]</code>, or <code>[Comment]</code> to submit a review with optional text.</li>
  <li>Select a merge method (<code>[Merge: merge commit]</code>, <code>[Merge: squash]</code>, or <code>[Merge: rebase]</code>) to merge.</li>
  <li>Use <code>[View Reviews]</code>, <code>[View Comments]</code>, or <code>[View Commits]</code> to inspect PR activity.</li>
</ol>

<h3 id="workflow-create-pr" align="center">Create a Pull Request</h3>
<ol>
  <li>Open a repository, then use <code>Ctrl+P</code> &gt; <code>Create Pull Request</code>.</li>
  <li>Follow the 4-step wizard: enter the PR title, head branch, base branch, and optional description.</li>
  <li>On completion, the PR is created via the GitHub API and the detail view is shown.</li>
</ol>

<h3 id="workflow-multi-select" align="center">Multi-Select Repositories</h3>
<ol>
  <li>In the repository list, navigate to a repo and press <code>Space</code> to select it.</li>
  <li>Selected repos show a <code>[*]</code> marker. Selected + active shows <code>&gt;[*]</code>.</li>
  <li>Press <code>Space</code> again to deselect.</li>
</ol>

<h2 id="troubleshooting" align="center">Troubleshooting</h2>
<ul>
  <li>If API limits are hit, set a token with <code>gitnapse auth set</code> or export <code>GITHUB_TOKEN</code>.</li>
  <li>For OAuth device flow, you can provide <code>--client-id</code>; if omitted, GitNapse uses env variables and then built-in default OAuth Client ID.</li>
  <li>You can also use <code>GITHUB_CLIENT_ID</code> as compatibility fallback for OAuth client ID.</li>
  <li>If OAuth URL is not clickable in your terminal, GitNapse still tries to auto-open browser; otherwise copy/open the displayed URL manually.</li>
  <li>To inspect your private repositories from TUI search, use <code>@me</code> (or <code>@me keyword</code> to filter).</li>
  <li>If token is saved but requests fail, run <code>gitnapse auth status</code> and validate token permissions.</li>
  <li>If clone/download fails, verify destination path permissions and filesystem access.</li>
  <li>If no repos appear, refine query terms (owner/org/repo keywords).</li>
  <li>If the command palette does not open with <code>Ctrl+P</code>, your terminal may intercept the key combination. Try a different terminal emulator.</li>
</ul>

<h3 id="macos-keychain-prompts" align="center">macOS: Keychain Password Prompts</h3>
<p>
  On macOS, GitNapse stores your GitHub token in the system Keychain via the <code>keyring</code>
  crate. You may see a dialog asking <em>"gitnapse wants to use the Keychain to store a secret"</em>
  or prompting for your login password each time you run GitNapse.
</p>
<p>To stop the prompts permanently:</p>
<ol>
  <li>Open <strong>Keychain Access</strong> app (from <code>/Applications/Utilities/</code> or Spotlight).</li>
  <li>Search for <code>com.GitNapse.GitNapse</code>.</li>
  <li>Double-click the entry, go to the <strong>Access Control</strong> tab.</li>
  <li>Select <strong>"Allow all applications to access this item"</strong>.</li>
  <li>Click <strong>Save Changes</strong> (you may be asked for your login password to confirm).</li>
</ol>
<p>
  Alternatively, export the <code>GITHUB_TOKEN</code> environment variable in your shell profile
  (<code>~/.zshrc</code>, <code>~/.bashrc</code>, etc.) to bypass the Keychain entirely:
</p>
<pre><code>export GITHUB_TOKEN="ghp_your_token_here"</code></pre>
