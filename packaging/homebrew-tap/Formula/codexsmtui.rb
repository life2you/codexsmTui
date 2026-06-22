class Codexsmtui < Formula
  desc "Terminal-first TUI for managing local OpenAI Codex CLI sessions"
  homepage "https://github.com/life2you/codexsmTui"
  version "0.1.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/life2you/codexsmTui/releases/download/v0.1.3/codexsmtui-aarch64-apple-darwin.tar.gz"
      sha256 "9138234cf57ce53a71915f6c15486e792171b5b886580b7f95b7e2174895da6b"
    end

    on_intel do
      url "https://github.com/life2you/codexsmTui/releases/download/v0.1.3/codexsmtui-x86_64-apple-darwin.tar.gz"
      sha256 "af778e9c0c8aea76c42c05d236c4f265fe24d70bc548abfa791dbc20cc7fce8b"
    end
  end

  def install
    bin.install "codexsmTui"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/codexsmTui --version")
  end
end
