class Portwatch < Formula
  desc "Terminal UI for monitoring and managing local web server ports"
  homepage "https://github.com/malazaysc/portwatch"
  version "0.3.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/malazaysc/portwatch/releases/download/v#{version}/portwatch-macos-aarch64.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/malazaysc/portwatch/releases/download/v#{version}/portwatch-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/malazaysc/portwatch/releases/download/v#{version}/portwatch-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/malazaysc/portwatch/releases/download/v#{version}/portwatch-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "portwatch"
  end

  test do
    assert_match "portwatch #{version}", shell_output("#{bin}/portwatch --version")
  end
end
