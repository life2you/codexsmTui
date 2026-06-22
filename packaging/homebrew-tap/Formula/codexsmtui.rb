class Codexsmtui < Formula
  desc "Terminal-first TUI for managing local OpenAI Codex CLI sessions"
  homepage "https://github.com/life2you/codexsmTui"
  version "0.1.4"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/life2you/codexsmTui/releases/download/v0.1.4/codexsmtui-aarch64-apple-darwin.tar.gz"
      sha256 "a1bc207156acea70e5674206bc3dd5f644f0df2c20fa95fce0be598437eeebb6"
    end

    on_intel do
      url "https://github.com/life2you/codexsmTui/releases/download/v0.1.4/codexsmtui-x86_64-apple-darwin.tar.gz"
      sha256 "96c8f567576d5608ef1e2a26a172997841c44fabc4b652a56d949c9252d66e0e"
    end
  end

  def install
    bin.install "codexsmTui"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/codexsmTui --version")
  end
end
