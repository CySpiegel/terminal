class Terminal < Formula
  desc "GPU-rendered terminal emulator with built-in GTD"
  homepage "https://github.com/CySpiegel/terminal"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CySpiegel/terminal/releases/download/v#{version}/terminal-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/CySpiegel/terminal/releases/download/v#{version}/terminal-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    url "https://github.com/CySpiegel/terminal/releases/download/v#{version}/terminal-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER"
  end

  def install
    bin.install "terminal"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/terminal --version")
  end
end
