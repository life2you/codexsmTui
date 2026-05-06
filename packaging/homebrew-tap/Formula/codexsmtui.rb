class CodexsmTui < Formula
  desc "Terminal-first TUI for managing local OpenAI Codex CLI sessions"
  homepage "https://github.com/life2you/codexsmTui"
  url "https://github.com/life2you/codexsmTui/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "364b515e8ae59904e2458357c5c92597b2be5462f47c6d9574eb97951cebd8a0"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/codexsmTui --version")
  end
end
