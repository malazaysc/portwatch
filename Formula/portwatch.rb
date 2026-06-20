class Portwatch < Formula
  desc "Terminal UI for monitoring and managing local web server ports"
  homepage "https://github.com/malazaysc/portwatch"
  version "0.4.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/malazaysc/portwatch/releases/download/v#{version}/portwatch-macos-aarch64.tar.gz"
      sha256 "876ec8398016298276b4804ccacd516935c1a2c7d53d0beb0885fcda8d009df2"
    else
      url "https://github.com/malazaysc/portwatch/releases/download/v#{version}/portwatch-macos-x86_64.tar.gz"
      sha256 "3ce0ee5e89b5dd02161f0b0c3390b1c4b8065c512f5b98ad91c2333d4cb23d39"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/malazaysc/portwatch/releases/download/v#{version}/portwatch-linux-aarch64.tar.gz"
      sha256 "58bd46cd5ceb3c4b633278a4a0d5ff86ec477ad4e1999ad35f0b1854a0389158"
    else
      url "https://github.com/malazaysc/portwatch/releases/download/v#{version}/portwatch-linux-x86_64.tar.gz"
      sha256 "8fe6c444b3445c4076d80817ec7dc9f788d5022360cd8afbfdecbe5a075566c5"
    end
  end

  def install
    bin.install "portwatch"
  end

  test do
    assert_match "portwatch #{version}", shell_output("#{bin}/portwatch --version")
  end
end
