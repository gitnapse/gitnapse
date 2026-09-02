<h1 align="center">Security Audit Guide</h1>

<h2 align="center">Automated Audit Locally</h2>
<p>
  GitNapse runs all checks locally via <code>scripts/ci.sh</code>. No remote CI gates are required.
</p>
<ul>
  <li><code>cargo fmt --all -- --check</code></li>
  <li><code>cargo clippy --all-targets --all-features -- -D warnings</code></li>
  <li><code>cargo test --all-targets --all-features</code></li>
  <li><code>cargo audit --ignore RUSTSEC-2023-0071</code></li>
</ul>

<h2 align="center">Local Audit Commands</h2>
<pre><code class="language-bash">cargo install cargo-audit --locked
cargo audit --ignore RUSTSEC-2023-0071
</code></pre>

<h2 align="center">Scope</h2>
<ul>
  <li>Dependency CVE scanning</li>
  <li>Static quality and lint hardening</li>
  <li>Regression checks on authentication and secure storage paths</li>
</ul>

<h2 align="center">Current Advisory Exception</h2>
<p>
  The advisory <code>RUSTSEC-2023-0071</code> is currently transitive through
  <code>octocrab -&gt; jsonwebtoken -&gt; rsa</code> and has no fixed upgrade available in the current dependency line.
  <code>scripts/ci.sh</code> keeps this ID explicitly ignored until upstream provides a fix.
</p>
