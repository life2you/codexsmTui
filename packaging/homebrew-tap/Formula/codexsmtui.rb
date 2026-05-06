class Codexsmtui < Formula
  desc "Terminal-first TUI for managing local OpenAI Codex CLI sessions"
  homepage "https://github.com/life2you/codexsmTui"
  url "https://github.com/life2you/codexsmTui/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "06e2e450c085cb0be1be63796bf47bb1ceca9bc61eb7c6bb73103b1edaf0cb1f"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/codexsmTui --version")
  end
end
