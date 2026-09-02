<h1 align="center">GitNapse Remote Installation</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#unix-script">Linux/macOS Remote Script (.sh)</a></li>
  <li><a href="#windows-script">Windows Remote Script (.ps1)</a></li>
  <li><a href="#examples">Command Examples</a></li>
  <li><a href="#security-notes">Security Notes</a></li>
</ul>

<h2 id="overview" align="center">Overview</h2>
<p>
  GitNapse includes remote installation scripts at the repository root so the tool can be installed
  or uninstalled without cloning the repository first.
</p>
<ul>
  <li><code>install.sh</code> - Linux/macOS installer/uninstaller with dependency checks and Rust bootstrap.</li>
  <li><code>install.ps1</code> - Windows 11 PowerShell installer/uninstaller with Rust bootstrap.</li>
</ul>

<h2 id="unix-script" align="center">Linux/macOS Remote Script (.sh)</h2>
<p>The shell script supports Linux and macOS with OS-aware preparation and Rust detection.</p>
<ul>
  <li>Linux package manager detection priority: <code>apt</code>, <code>pacman</code>, <code>dnf</code>, <code>yum</code>, <code>zypper</code>, fallback notice for <code>rpm</code>.</li>
  <li>macOS flow includes Xcode Command Line Tools check and Homebrew-based dependency installation.</li>
  <li>Checks for Rust (<code>cargo</code>/<code>rustc</code>) and installs Rustup only when missing.</li>
  <li>Installs GitNapse via <code>cargo install --git ... --locked gitnapse</code>.</li>
  <li>Supports uninstall and optional config/cache cleanup.</li>
</ul>

<p><strong>Linux script parameters:</strong></p>
<table>
  <thead>
    <tr>
      <th>Parameter</th>
      <th>Accepted Values</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr><td><code>--action</code></td><td><code>install</code> or <code>uninstall</code></td><td>Select operation mode.</td></tr>
    <tr><td><code>--repo-url</code></td><td>Git URL</td><td>Override repository source.</td></tr>
    <tr><td><code>--sudo</code></td><td>flag</td><td>Force <code>sudo</code> for package operations.</td></tr>
    <tr><td><code>--no-sudo</code></td><td>flag</td><td>Disable <code>sudo</code> usage on Linux.</td></tr>
    <tr><td><code>--cleanup</code></td><td>flag</td><td>On uninstall, remove local GitNapse config/cache.</td></tr>
  </tbody>
</table>

<h2 id="windows-script" align="center">Windows Remote Script (.ps1)</h2>
<p>The PowerShell script is designed for Windows 11 and supports install/uninstall with parameters.</p>
<ul>
  <li>Checks for <code>cargo</code> first and reuses existing Rust toolchain if present.</li>
  <li>If Rust is missing, installs Rustup via <code>winget</code> when available, or via direct rustup-init download fallback.</li>
  <li>Installs GitNapse from the Git repository using Cargo.</li>
  <li>Supports uninstall and optional cleanup of local app directories.</li>
</ul>

<p><strong>PowerShell script parameters:</strong></p>
<table>
  <thead>
    <tr>
      <th>Parameter</th>
      <th>Accepted Values</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr><td><code>-Action</code></td><td><code>install</code> or <code>uninstall</code></td><td>Select operation mode.</td></tr>
    <tr><td><code>-RepoUrl</code></td><td>Git URL</td><td>Override repository source.</td></tr>
    <tr><td><code>-Cleanup</code></td><td>switch</td><td>On uninstall, remove local GitNapse config/cache.</td></tr>
  </tbody>
</table>

<h2 id="examples" align="center">Command Examples</h2>
<p><strong>Linux/macOS install (curl):</strong></p>
<pre><code class="language-bash">curl -fsSL https://raw.githubusercontent.com/xscriptor/gitnapse/main/install.sh | bash -s -- --action install
</code></pre>

<p><strong>Linux/macOS uninstall with cleanup (wget):</strong></p>
<pre><code class="language-bash">wget -qO- https://raw.githubusercontent.com/xscriptor/gitnapse/main/install.sh | bash -s -- --action uninstall --cleanup
</code></pre>

<p><strong>Windows 11 install (PowerShell):</strong></p>
<pre><code class="language-powershell">irm https://raw.githubusercontent.com/xscriptor/gitnapse/main/install.ps1 | iex
</code></pre>

<p><strong>Windows 11 uninstall with cleanup (PowerShell):</strong></p>
<pre><code class="language-powershell">&amp; ([scriptblock]::Create((irm https://raw.githubusercontent.com/xscriptor/gitnapse/main/install.ps1))) -Action uninstall -Cleanup
</code></pre>

<h2 id="security-notes" align="center">Security Notes</h2>
<ul>
  <li>Review remote scripts before execution in sensitive environments.</li>
  <li>Prefer pinning to a specific commit in production automation.</li>
  <li>Run with least required privileges when possible.</li>
</ul>
