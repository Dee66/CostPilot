class Costpilot < Formula
  desc "Cost analysis and prediction for infrastructure as code"
  homepage "https://costpilot.dev"
  url "https://github.com/Dee66/CostPilot/releases/download/v1.0.0/costpilot-darwin-x86_64.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Dee66/CostPilot/releases/download/v#{version}/costpilot-darwin-aarch64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_ARM"
    else
      url "https://github.com/Dee66/CostPilot/releases/download/v#{version}/costpilot-darwin-x86_64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_X64"
    end
  end

  def install
    bin.install "costpilot"
  end

  test do
    system "#{bin}/costpilot", "--version"
  end
end