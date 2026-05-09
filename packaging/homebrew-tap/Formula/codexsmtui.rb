class Codexsmtui < Formula
  desc "Terminal-first TUI for managing local OpenAI Codex CLI sessions"
  homepage "https://github.com/life2you/codexsmTui"
  version "0.1.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/life2you/codexsmTui/releases/download/v0.1.1/codexsmtui-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    end

    on_intel do
      url "https://github.com/life2you/codexsmTui/releases/download/v0.1.1/codexsmtui-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X64_SHA256"
    end
  end

  def install
    bin.install "codexsmTui"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/codexsmTui --version")
  end
end
