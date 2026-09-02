<h1 align="center">GitNapse CLI Reference</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#authentication">Authentication</a></li>
  <li><a href="#git-operations">Git Operations (local git)</a></li>
  <li><a href="#api-operations">GitHub API Operations</a></li>
  <li><a href="#theme-registry">Theme Registry</a></li>
  <li><a href="#auto-detect">Auto-detect Repository</a></li>
  <li><a href="#error-handling">Error Handling</a></li>
  <li><a href="#examples">Quick Examples</a></li>
  <li><a href="#options-reference">Options Reference</a></li>
  <li><a href="#full-command-list">Full Command List</a></li>
</ul>

<h2 id="overview" align="center">Overview</h2>
  <p>
    GitNapse provides a set of CLI commands that operate both via the <strong>GitHub REST API</strong>
    (for queries, PR management, issues, CI checks, comparisons, releases, repo creation, search)
    and via <strong>local git</strong>
    (for clone, commit, push, pull, fetch, checkout, diff, stash, tag, reset, status, log, branch,
    remote, config, merge).
    It also manages UI themes through the <strong>remote theme registry</strong>.
    Authentication is shared across all GitHub API commands.
  </p>

<h2 id="authentication" align="center">Authentication</h2>
<p>
  Commands that hit the GitHub API (<code>clone</code> with <code>owner/repo</code> format,
  <code>pr</code>, <code>issue</code>, <code>ci</code>, <code>compare</code>,
  <code>release</code>, <code>repo</code>, <code>search</code>) resolve
  credentials in this order:
</p>
<ol>
  <li><code>GITHUB_TOKEN</code> environment variable</li>
  <li>OAuth session (from <code>gitnapse auth oauth login</code>)</li>
  <li>Stored token (from <code>gitnapse auth set</code>)</li>
</ol>

<h2 id="git-operations" align="center">Git Operations (local git)</h2>
<p>
  These commands run <code>git</code> locally. They must be executed from inside a git repository
  (except <code>clone</code>).
</p>

<table>
  <thead>
    <tr>
      <th>Command</th>
      <th>Purpose</th>
      <th>Example</th>
      <th>Notes</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>gitnapse clone</code></td>
      <td>Clone a repository</td>
      <td><code>gitnapse clone xscriptor/gitnapse:develop --dir ./myclone</code></td>
      <td>
        Accepts <code>owner/repo[:branch]</code> (resolved via API) or a full git URL.
        Optional <code>--dir</code> for destination. Checks if target already exists.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse commit</code></td>
      <td>Commit changes</td>
      <td><code>gitnapse commit -m "fix: typo" -a</code></td>
      <td>
        <code>-m</code> is required. <code>-a</code> stages all changes first
        (<code>git add -A</code>). Without <code>-a</code>, commits only staged changes.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse push</code></td>
      <td>Push commits to remote</td>
      <td><code>gitnapse push origin main --force-with-lease</code></td>
      <td>
        Optional <code>[remote]</code> and <code>[branch]</code>.
        <code>--force-with-lease</code> for safe force push.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse pull</code></td>
      <td>Pull changes from remote</td>
      <td><code>gitnapse pull --rebase</code></td>
      <td>
        Optional <code>[remote]</code> and <code>[branch]</code>.
        <code>--rebase</code> to rebase instead of merge.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse fetch</code></td>
      <td>Fetch from remote</td>
      <td><code>gitnapse fetch --prune</code></td>
      <td><code>--prune</code> removes stale remote-tracking branches.</td>
    </tr>
    <tr>
      <td><code>gitnapse checkout</code></td>
      <td>Switch branches</td>
      <td><code>gitnapse checkout -b feature/new</code></td>
      <td><code>-b</code> creates a new branch before switching.</td>
    </tr>
    <tr>
      <td><code>gitnapse merge</code></td>
      <td>Merge a branch into current</td>
      <td><code>gitnapse merge feature/new</code></td>
      <td>Merges the specified branch into the current branch.</td>
    </tr>
    <tr>
      <td><code>gitnapse diff</code></td>
      <td>Show working tree diff</td>
      <td><code>gitnapse diff --staged --path src/main.rs</code></td>
      <td><code>--staged</code> shows staged changes. <code>--path</code> filters by file.</td>
    </tr>
    <tr>
      <td><code>gitnapse remote list</code></td>
      <td>List remotes</td>
      <td><code>gitnapse remote list</code></td>
      <td>Shows <code>git remote -v</code> output.</td>
    </tr>
    <tr>
      <td><code>gitnapse remote add</code></td>
      <td>Add a remote</td>
      <td><code>gitnapse remote add origin https://github.com/user/repo.git</code></td>
      <td>Adds a new remote URL.</td>
    </tr>
    <tr>
      <td><code>gitnapse remote remove</code></td>
      <td>Remove a remote</td>
      <td><code>gitnapse remote remove origin</code></td>
      <td>Removes a remote by name.</td>
    </tr>
    <tr>
      <td><code>gitnapse remote rename</code></td>
      <td>Rename a remote</td>
      <td><code>gitnapse remote rename origin upstream</code></td>
      <td>Renames a remote from old name to new name.</td>
    </tr>
    <tr>
      <td><code>gitnapse config get</code></td>
      <td>Get a config value</td>
      <td><code>gitnapse config get user.name</code></td>
      <td>Shows the value of a git config key.</td>
    </tr>
    <tr>
      <td><code>gitnapse config set</code></td>
      <td>Set a config value</td>
      <td><code>gitnapse config set user.name "Your Name"</code></td>
      <td>Sets a git config key to a value.</td>
    </tr>
    <tr>
      <td><code>gitnapse config list</code></td>
      <td>List all config</td>
      <td><code>gitnapse config list</code></td>
      <td>Shows <code>git config --list</code> output.</td>
    </tr>
    <tr>
      <td><code>gitnapse stash push</code></td>
      <td>Stash changes</td>
      <td><code>gitnapse stash push -m "WIP"</code></td>
      <td><code>-m</code> adds a description to the stash.</td>
    </tr>
    <tr>
      <td><code>gitnapse stash pop</code></td>
      <td>Restore topmost stash</td>
      <td><code>gitnapse stash pop</code></td>
      <td>Restores and removes the latest stash.</td>
    </tr>
    <tr>
      <td><code>gitnapse stash list</code></td>
      <td>List stashes</td>
      <td><code>gitnapse stash list</code></td>
      <td>Shows all stashed entries.</td>
    </tr>
    <tr>
      <td><code>gitnapse tag list</code></td>
      <td>List tags</td>
      <td><code>gitnapse tag list "v*"</code></td>
      <td>Optional glob pattern to filter.</td>
    </tr>
    <tr>
      <td><code>gitnapse tag create</code></td>
      <td>Create a tag</td>
      <td><code>gitnapse tag create v1.0 -m "Release 1.0"</code></td>
      <td><code>-m</code> creates an annotated tag.</td>
    </tr>
    <tr>
      <td><code>gitnapse tag delete</code></td>
      <td>Delete a tag (local + remote)</td>
      <td><code>gitnapse tag delete v1.0</code></td>
      <td>Deletes locally and pushes the deletion to remote.</td>
    </tr>
    <tr>
      <td><code>gitnapse status</code></td>
      <td>Show working tree status</td>
      <td><code>gitnapse status</code></td>
      <td>Runs <code>git status --short</code>. Shows <code>(clean)</code> when nothing changed.</td>
    </tr>
    <tr>
      <td><code>gitnapse log</code></td>
      <td>Show commit log</td>
      <td><code>gitnapse log -n 10</code></td>
      <td>Runs <code>git log --oneline -n</code>. Default: 20 entries.</td>
    </tr>
    <tr>
      <td><code>gitnapse branch</code></td>
      <td>List branches</td>
      <td><code>gitnapse branch</code></td>
      <td>Runs <code>git branch -a</code>.</td>
    </tr>
    <tr>
      <td><code>gitnapse reset</code></td>
      <td>Reset current HEAD</td>
      <td><code>gitnapse reset HEAD~1 --hard</code></td>
      <td>Target defaults to <code>HEAD</code>. <code>--hard</code> discards working tree changes.</td>
    </tr>
  </tbody>
</table>

<h2 id="api-operations" align="center">GitHub API Operations</h2>
<p>
  These commands use the GitHub REST API and require authentication for private repositories
  or for creating/merging PRs and issues.
</p>

<table>
  <thead>
    <tr>
      <th>Command</th>
      <th>Purpose</th>
      <th>Example</th>
      <th>Notes</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>gitnapse pr list</code></td>
      <td>List pull requests</td>
      <td><code>gitnapse pr list xscriptor/gitnapse -s open</code></td>
      <td>
        Shows PR number, state, additions/deletions, title, and author.
        State: <code>open</code> (default), <code>closed</code>, <code>all</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse pr create</code></td>
      <td>Create a pull request</td>
      <td><code>gitnapse pr create xscriptor/gitnapse -t "Feature" -H feat/new -B main</code></td>
      <td>
        Requires <code>--title</code>, <code>--head</code> (source), <code>--base</code> (target).
        Optional <code>--body</code> for description.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse pr merge</code></td>
      <td>Merge a pull request</td>
      <td><code>gitnapse pr merge xscriptor/gitnapse -n 42 -m squash</code></td>
      <td>
        <code>--number</code> required. Method: <code>merge</code> (default),
        <code>squash</code>, or <code>rebase</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse issue list</code></td>
      <td>List issues</td>
      <td><code>gitnapse issue list xscriptor/gitnapse -s open</code></td>
      <td>
        Shows issue number, state, title, and author. PRs show a <code>PR</code> marker.
        State: <code>open</code> (default), <code>closed</code>, <code>all</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse issue create</code></td>
      <td>Create an issue</td>
      <td><code>gitnapse issue create xscriptor/gitnapse -t "Bug" -b "description"</code></td>
      <td><code>--title</code> required. Optional <code>--body</code>.</td>
    </tr>
    <tr>
      <td><code>gitnapse issue close</code></td>
      <td>Close an issue</td>
      <td><code>gitnapse issue close xscriptor/gitnapse -n 42</code></td>
      <td>Requires <code>--number</code>.</td>
    </tr>
    <tr>
      <td><code>gitnapse ci</code></td>
      <td>Show CI status</td>
      <td><code>gitnapse ci xscriptor/gitnapse -b main</code></td>
      <td>
        Shows check runs for the latest commit on the branch.
        Branch defaults to <code>main</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse compare</code></td>
      <td>Compare two branches</td>
      <td><code>gitnapse compare xscriptor/gitnapse main develop</code></td>
      <td>Shows ahead/behind count, files changed with additions and deletions per file.</td>
    </tr>
    <tr>
      <td><code>gitnapse release list</code></td>
      <td>List releases</td>
      <td><code>gitnapse release list xscriptor/gitnapse</code></td>
      <td>Shows tag name, release title, and pre-release status.</td>
    </tr>
    <tr>
      <td><code>gitnapse release create</code></td>
      <td>Create a release</td>
      <td><code>gitnapse release create xscriptor/gitnapse v1.0 -n "v1.0" -b "changelog" --prerelease</code></td>
      <td>
        Requires tag name. Options: <code>--name</code>, <code>--body</code>,
        <code>--prerelease</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse repo create</code></td>
      <td>Create a repository</td>
      <td><code>gitnapse repo create my-project -d "description" -p</code></td>
      <td>
        <code>--description</code> optional. <code>--private</code> for private repos.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse search</code></td>
      <td>Search repositories</td>
      <td><code>gitnapse search "rust language:wasm"</code></td>
      <td>Uses GitHub search API. Shows stars, language, and description.</td>
    </tr>
  </tbody>
</table>

<h2 id="theme-registry" align="center">Theme Registry</h2>
<p>
  Theme management reads the remote registry at <code>github.com/gitnapse/themes</code>.
  The registry is a public data repo (no authentication required): <code>index.json</code>
  at its root lists available themes and where each file lives. Installed themes are
  written to the config directory's <code>themes/</code> folder.
</p>

<table>
  <thead>
    <tr>
      <th>Command</th>
      <th>Purpose</th>
      <th>Example</th>
      <th>Notes</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>gitnapse theme list</code></td>
      <td>List installed themes</td>
      <td><code>gitnapse theme list</code></td>
      <td>Add <code>--remote</code> to also query the registry index.</td>
    </tr>
    <tr>
      <td><code>gitnapse theme install</code></td>
      <td>Install a theme from the registry</td>
      <td><code>gitnapse theme install Tokio</code></td>
      <td>Requires the registry to be reachable. Theme names are validated before any file is written.</td>
    </tr>
    <tr>
      <td><code>gitnapse theme uninstall</code></td>
      <td>Remove an installed theme</td>
      <td><code>gitnapse theme uninstall Tokio</code></td>
      <td>Removes <code>&lt;name&gt;.jsonc</code> from the config themes folder.</td>
    </tr>
  </tbody>
</table>

<h2 id="auto-detect" align="center">Auto-detect Repository</h2>
<p>
  When using API commands (<code>pr</code>, <code>issue</code>, <code>ci</code>,
  <code>compare</code>, <code>release</code>), you can omit <code>owner/</code> if you are
  inside a cloned repository — GitNapse will parse the <code>origin</code> remote to extract
  <code>owner/repo</code> automatically.
</p>

<pre><code># Inside /home/user/projects/gitnapse (cloned from xscriptor/gitnapse)
gitnapse pr list            # works — detects xscriptor/gitnapse
gitnapse issue list         # works
gitnapse ci                 # works

# Outside a cloned repo, you must specify:
gitnapse pr list xscriptor/gitnapse
</code></pre>

<h2 id="error-handling" align="center">Error Handling</h2>
<p>
  All CLI commands provide user-friendly error messages instead of raw Rust traces:
</p>

<table>
  <thead>
    <tr>
      <th>Situation</th>
      <th>Message</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>CWD is not a git repository</td>
      <td><code>not a git repository — run this from inside a git repository</code></td>
    </tr>
    <tr>
      <td>Repository not found on GitHub</td>
      <td><code>repository 'owner/repo' not found on GitHub</code></td>
    </tr>
    <tr>
      <td>Authentication required</td>
      <td><code>authentication required — run 'gitnapse auth set' or 'gitnapse auth oauth login'</code></td>
    </tr>
    <tr>
      <td>Git not installed</td>
      <td><code>git is not installed or not in PATH</code> with install link</td>
    </tr>
    <tr>
      <td>Empty commit message</td>
      <td><code>commit message cannot be empty</code> with usage hint</td>
    </tr>
    <tr>
      <td>Empty repository spec</td>
      <td><code>repository specification is empty</code> with usage hint</td>
    </tr>
    <tr>
      <td>Destination path exists</td>
      <td><code>destination path '...' already exists</code></td>
    </tr>
    <tr>
      <td>Nothing to commit</td>
      <td><code>nothing to commit (working tree clean)</code></td>
    </tr>
    <tr>
      <td>No stash entries</td>
      <td><code>no stash entries to pop</code></td>
    </tr>
    <tr>
      <td>GitHub API rate limit</td>
      <td><code>GitHub API rate limit exceeded — resets at timestamp ...</code></td>
    </tr>
  </tbody>
</table>

<h2 id="examples" align="center">Quick Examples</h2>

<pre><code># Clone a repo
gitnapse clone xscriptor/gitnapse
gitnapse clone xscriptor/gitnapse:develop --dir ./myclone
gitnapse clone https://github.com/xscriptor/gitnapse.git

# Daily workflow
gitnapse pull --rebase
gitnapse status
gitnapse diff --staged
gitnapse commit -m "feat: add login" -a
gitnapse push origin main

# Branches, merge, remotes
gitnapse checkout -b feature/new
gitnapse merge feature/new
gitnapse branch
gitnapse log -n 10
gitnapse remote add upstream https://github.com/other/repo.git
gitnapse remote list

# Config
gitnapse config get user.name
gitnapse config set user.email "me@example.com"
gitnapse config list

# Stashing
gitnapse stash push -m "WIP before refactor"
gitnapse stash list
gitnapse stash pop

# Tags
gitnapse tag list "v*"
gitnapse tag create v1.0 -m "Release 1.0"
gitnapse tag delete v0.9

# Reset
gitnapse reset HEAD~1
gitnapse reset HEAD~2 --hard

# Pull request management via API
gitnapse pr list
gitnapse pr create -t "Feature" -H feat/new -B main
gitnapse pr merge -n 7 -m squash

# Issues via API
gitnapse issue list -s open
gitnapse issue create -t "Bug found" -b "details here"
gitnapse issue close -n 42

# CI and comparison
gitnapse ci xscriptor/gitnapse -b main
gitnapse compare xscriptor/gitnapse main develop

# Releases, repo creation, search
gitnapse release list xscriptor/gitnapse
gitnapse release create xscriptor/gitnapse v1.0 -n "v1.0" -b "changelog"
gitnapse repo create new-project -d "My new project" -p
gitnapse search "rust language:wasm"

# Theme registry
gitnapse theme list
gitnapse theme list --remote
gitnapse theme install Tokio
gitnapse theme uninstall Tokio
</code></pre>

<h2 id="options-reference" align="center">Options Reference (per command)</h2>
<p>
  The reference below is generated from the CLI itself: each block is the exact
  output of <code>gitnapse &lt;command&gt; --help</code>, so the descriptions
  always match the real behavior. Regenerate whenever options change.
</p>

<h3>gitnapse run</h3>
<pre><code>
Run interactive terminal UI

Usage: gitnapse run [OPTIONS]

Options:
      --query <QUERY>                    Initial search query [default: ""]
      --page <PAGE>                      Search results page to load [default: 1]
      --per-page <PER_PAGE>              Results per page (max 100) [default: 30]
      --cache-ttl-secs <CACHE_TTL_SECS>  Preview cache TTL in seconds [default: 900]
  -h, --help                             Print help
</code></pre>

<h3>gitnapse download-file</h3>
<pre><code>
Download one file from a GitHub repository (curl/wget-like)

Usage: gitnapse download-file [OPTIONS] --repo <REPO> --path <PATH> --out <OUT>

Options:
      --repo <REPO>  Repository in owner/name form
      --path <PATH>  Path of the file inside the repository
      --ref <REF>    Branch/tag/commit (default: HEAD)
      --out <OUT>    Destination local file path
  -h, --help         Print help
</code></pre>

<h3>gitnapse clone</h3>
<pre><code>
Clone a repository (via API + git)

Usage: gitnapse clone [OPTIONS] <REPO>

Arguments:
  <REPO>  owner/repo[:branch] or a full git URL

Options:
      --dir <DIR>  Destination directory (default: repository name in the current dir)
  -h, --help       Print help
</code></pre>

<h3>gitnapse commit</h3>
<pre><code>
Stage (with -a) and commit changes

Usage: gitnapse commit [OPTIONS] -m <MESSAGE>

Options:
  -m <MESSAGE>  Commit message
  -a            Stage all changes first (git add -A)
  -h, --help    Print help
</code></pre>

<h3>gitnapse push</h3>
<pre><code>
Push commits to remote

Usage: gitnapse push [OPTIONS] [REMOTE] [BRANCH]

Arguments:
  [REMOTE]  Remote name (default: origin)
  [BRANCH]  Branch to push (default: current branch)

Options:
      --force-with-lease  Force push with --force-with-lease
  -h, --help              Print help
</code></pre>

<h3>gitnapse pull</h3>
<pre><code>
Pull changes from remote (with --rebase)

Usage: gitnapse pull [OPTIONS] [REMOTE] [BRANCH]

Arguments:
  [REMOTE]  Remote name (default: origin)
  [BRANCH]  Branch to pull (default: current branch)

Options:
      --rebase  Rebase instead of merge
  -h, --help    Print help
</code></pre>

<h3>gitnapse fetch</h3>
<pre><code>
Fetch from remote (with --prune)

Usage: gitnapse fetch [OPTIONS]

Options:
      --prune  Remove stale remote-tracking branches
  -h, --help   Print help
</code></pre>

<h3>gitnapse checkout</h3>
<pre><code>
Switch branches or restore files

Usage: gitnapse checkout [OPTIONS] <BRANCH>

Arguments:
  <BRANCH>  Branch to switch to

Options:
  -b          Create the branch before switching
  -h, --help  Print help
</code></pre>

<h3>gitnapse diff</h3>
<pre><code>
Show working tree diff

Usage: gitnapse diff [OPTIONS]

Options:
      --staged       Show staged changes instead of unstaged
      --path <PATH>  Restrict the diff to a single path
  -h, --help         Print help
</code></pre>

<h3>gitnapse status</h3>
<pre><code>
Show working tree status

Usage: gitnapse status

Options:
  -h, --help  Print help
</code></pre>

<h3>gitnapse log</h3>
<pre><code>
Show commit log (default: 20 entries)

Usage: gitnapse log [OPTIONS]

Options:
  -n <COUNT>  Number of commits to show [default: 20]
  -h, --help  Print help
</code></pre>

<h3>gitnapse branch</h3>
<pre><code>
List branches

Usage: gitnapse branch

Options:
  -h, --help  Print help
</code></pre>

<h3>gitnapse reset</h3>
<pre><code>
Reset current HEAD

Usage: gitnapse reset [OPTIONS] [TARGET]

Arguments:
  [TARGET]  Ref to reset to (default: HEAD)

Options:
      --hard  Discard working tree changes
  -h, --help  Print help
</code></pre>

<h3>gitnapse merge</h3>
<pre><code>
Merge a branch into current

Usage: gitnapse merge <BRANCH>

Arguments:
  <BRANCH>  Branch to merge into the current branch

Options:
  -h, --help  Print help
</code></pre>

<h3>gitnapse search</h3>
<pre><code>
Search repositories on GitHub

Usage: gitnapse search <QUERY>

Arguments:
  <QUERY>  Search query

Options:
  -h, --help  Print help
</code></pre>

<h3>gitnapse theme</h3>
<pre><code>
List, install or remove themes from the registry

Usage: gitnapse theme <COMMAND>

Commands:
  list       List installed themes (add --remote to query the registry)
  install    Install a theme from the registry (github.com/gitnapse/themes)
  uninstall  Remove an installed theme
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
</code></pre>

<h3>gitnapse theme list</h3>
<pre><code>
error: unrecognized subcommand 'theme list'

  tip: a similar subcommand exists: 'theme'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse theme install</h3>
<pre><code>
error: unrecognized subcommand 'theme install'

  tip: a similar subcommand exists: 'theme'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse theme uninstall</h3>
<pre><code>
error: unrecognized subcommand 'theme uninstall'

  tip: a similar subcommand exists: 'theme'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse stash push</h3>
<pre><code>
error: unrecognized subcommand 'stash push'

  tip: some similar subcommands exist: 'status', 'stash'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse stash pop</h3>
<pre><code>
error: unrecognized subcommand 'stash pop'

  tip: some similar subcommands exist: 'status', 'stash'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse stash list</h3>
<pre><code>
error: unrecognized subcommand 'stash list'

  tip: a similar subcommand exists: 'stash'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse tag list</h3>
<pre><code>
error: unrecognized subcommand 'tag list'

  tip: a similar subcommand exists: 'tag'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse tag create</h3>
<pre><code>
error: unrecognized subcommand 'tag create'

  tip: a similar subcommand exists: 'tag'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse tag delete</h3>
<pre><code>
error: unrecognized subcommand 'tag delete'

  tip: a similar subcommand exists: 'tag'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse pr list</h3>
<pre><code>
error: unrecognized subcommand 'pr list'

  tip: a similar subcommand exists: 'pr'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse pr create</h3>
<pre><code>
error: unrecognized subcommand 'pr create'

  tip: a similar subcommand exists: 'pr'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse pr merge</h3>
<pre><code>
error: unrecognized subcommand 'pr merge'

  tip: some similar subcommands exist: 'pr', 'merge'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse issue list</h3>
<pre><code>
error: unrecognized subcommand 'issue list'

  tip: a similar subcommand exists: 'issue'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse issue create</h3>
<pre><code>
error: unrecognized subcommand 'issue create'

  tip: a similar subcommand exists: 'issue'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse issue close</h3>
<pre><code>
error: unrecognized subcommand 'issue close'

  tip: a similar subcommand exists: 'issue'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse ci</h3>
<pre><code>
Show CI status for a repository

Usage: gitnapse ci [OPTIONS] <REPO>

Arguments:
  <REPO>  Repository in owner/name form (auto-detected inside a clone)

Options:
  -b, --branch <BRANCH>  Branch to inspect (default: main)
  -w, --workflows        Also list workflow runs
  -h, --help             Print help
</code></pre>

<h3>gitnapse compare</h3>
<pre><code>
Compare two branches

Usage: gitnapse compare <REPO> <BASE> <HEAD>

Arguments:
  <REPO>  Repository in owner/name form (auto-detected inside a clone)
  <BASE>  Base branch
  <HEAD>  Head branch

Options:
  -h, --help  Print help
</code></pre>

<h3>gitnapse remote list</h3>
<pre><code>
error: unrecognized subcommand 'remote list'

  tip: a similar subcommand exists: 'remote'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse remote add</h3>
<pre><code>
error: unrecognized subcommand 'remote add'

  tip: a similar subcommand exists: 'remote'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse remote remove</h3>
<pre><code>
error: unrecognized subcommand 'remote remove'

  tip: a similar subcommand exists: 'remote'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse remote rename</h3>
<pre><code>
error: unrecognized subcommand 'remote rename'

  tip: a similar subcommand exists: 'remote'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse config get</h3>
<pre><code>
error: unrecognized subcommand 'config get'

  tip: some similar subcommands exist: 'clone', 'ci', 'config'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse config set</h3>
<pre><code>
error: unrecognized subcommand 'config set'

  tip: some similar subcommands exist: 'clone', 'ci', 'config'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse config list</h3>
<pre><code>
error: unrecognized subcommand 'config list'

  tip: some similar subcommands exist: 'ci', 'config'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse release list</h3>
<pre><code>
error: unrecognized subcommand 'release list'

  tip: a similar subcommand exists: 'release'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse release create</h3>
<pre><code>
error: unrecognized subcommand 'release create'

  tip: a similar subcommand exists: 'release'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse repo create</h3>
<pre><code>
error: unrecognized subcommand 'repo create'

  tip: some similar subcommands exist: 'release', 'repo'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse auth set</h3>
<pre><code>
error: unrecognized subcommand 'auth set'

  tip: a similar subcommand exists: 'auth'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse auth clear</h3>
<pre><code>
error: unrecognized subcommand 'auth clear'

  tip: a similar subcommand exists: 'auth'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse auth status</h3>
<pre><code>
error: unrecognized subcommand 'auth status'

  tip: a similar subcommand exists: 'auth'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse auth oauth login</h3>
<pre><code>
error: unrecognized subcommand 'auth oauth login'

  tip: a similar subcommand exists: 'auth'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>

<h3>gitnapse auth oauth status</h3>
<pre><code>
error: unrecognized subcommand 'auth oauth status'

  tip: a similar subcommand exists: 'auth'

Usage: gitnapse [COMMAND]

For more information, try '--help'.
</code></pre>


<h2 id="full-command-list" align="center">Full Command List</h2>

<pre><code>gitnapse
├── run              TUI with options
├── download-file    Download a file from GitHub
├── auth             Token and OAuth management
├── clone            Clone a repository (API + git)
├── commit           Commit changes (with -a for all)
├── push             Push to remote (--force-with-lease)
├── pull             Pull from remote (--rebase)
├── fetch            Fetch from remote (--prune)
├── checkout         Switch branches (-b to create)
├── merge            Merge a branch into current
├── diff             Show diff (--staged, --path)
├── remote
│   ├── list         List remotes
│   ├── add          Add a remote
│   ├── remove       Remove a remote
│   └── rename       Rename a remote
├── config
│   ├── get          Get a config value
│   ├── set          Set a config value
│   └── list         List all config
├── stash
│   ├── push         Stash changes
│   ├── pop          Restore topmost stash
│   └── list         List stashes
├── tag
│   ├── list         List tags
│   ├── create       Create a tag
│   └── delete       Delete a tag (local + remote)
├── status           Working tree status
├── log              Commit log (-n)
├── branch           List branches
├── reset            Reset HEAD (--hard)
├── pr
│   ├── list         List PRs
│   ├── create       Create a PR
│   └── merge        Merge a PR
├── issue
│   ├── list         List issues
│   ├── create       Create an issue
│   └── close        Close an issue
├── ci               CI status for a branch
├── compare          Compare two branches
├── release
│   ├── list         List releases
│   └── create       Create a release
├── repo
│   └── create       Create a repository
├── theme
│   ├── list         List installed themes (--remote to query the registry)
│   ├── install      Install a theme from the registry
│   └── uninstall    Remove an installed theme
└── search           Search repositories on GitHub
</code></pre>
