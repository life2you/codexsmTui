class Codexsmtui < Formula
  desc "Terminal-first TUI for managing local OpenAI Codex CLI sessions"
  homepage "https://github.com/life2you/codexsmTui"
  version "0.1.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/life2you/codexsmTui/releases/download/v0.1.2/codexsmtui-aarch64-apple-darwin.tar.gz"
      sha256 "7c9ca9131e116aaff292011ae27dbeb154edffece28aa67898ad0f4b2a24018d"
    end

    on_intel do
      url "https://github.com/life2you/codexsmTui/releases/download/v0.1.2/codexsmtui-x86_64-apple-darwin.tar.gz"
      sha256 "b03171560cbe5fca4638a525beeb2696076171fc7a6d65154956b919a1bf2900"
    end
  end

  def install
    bin.install "codexsmTui"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/codexsmTui --version")
  end
end
