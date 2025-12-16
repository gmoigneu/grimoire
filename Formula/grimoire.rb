# Homebrew Formula for GRIMOIRE
#
# Users can install with:
#   brew tap gmoigneu/grimoire
#   brew install grimoire
#
# Update the version and sha256 hashes for each release

class Grimoire < Formula
  desc "TUI for managing prompts, agents, and skills configuration"
  homepage "https://github.com/gmoigneu/grimoire"
  license "Apache-2.0"
  version "0.1.0"

  on_macos do
    on_intel do
      url "https://github.com/gmoigneu/grimoire/releases/download/v#{version}/grimoire-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end

    on_arm do
      url "https://github.com/gmoigneu/grimoire/releases/download/v#{version}/grimoire-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/gmoigneu/grimoire/releases/download/v#{version}/grimoire-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end

    on_arm do
      url "https://github.com/gmoigneu/grimoire/releases/download/v#{version}/grimoire-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  def install
    bin.install "grimoire"
  end

  test do
    assert_match "grimoire", shell_output("#{bin}/grimoire --version")
  end
end
