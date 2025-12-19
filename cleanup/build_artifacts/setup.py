from setuptools import setup
import os
import platform
import urllib.request
import subprocess

# Download the appropriate binary based on platform
def download_binary():
    system = platform.system().lower()
    machine = platform.machine().lower()

    # Map platform names
    if system == "linux":
        os_name = "linux"
    elif system == "darwin":
        os_name = "darwin"
    elif system == "windows":
        os_name = "windows"
    else:
        raise RuntimeError(f"Unsupported platform: {system}")

    if machine in ["x86_64", "amd64"]:
        arch_name = "x86_64"
    elif machine in ["arm64", "aarch64"]:
        arch_name = "aarch64"
    else:
        raise RuntimeError(f"Unsupported architecture: {machine}")

    version = "1.0.0"
    binary_name = f"costpilot-{os_name}-{arch_name}"
    if system == "windows":
        binary_name += ".exe"

    url = f"https://github.com/Dee66/CostPilot/releases/download/v{version}/{binary_name}"

    # Create bin directory if it doesn't exist
    os.makedirs("bin", exist_ok=True)
    binary_path = os.path.join("bin", "costpilot.exe" if system == "windows" else "costpilot")

    print(f"Downloading CostPilot from {url}")
    urllib.request.urlretrieve(url, binary_path)

    # Make executable on Unix-like systems
    if system != "windows":
        os.chmod(binary_path, 0o755)

    return binary_path

# Download binary during setup
try:
    binary_path = os.path.join("bin", "costpilot.exe" if system == "windows" else "costpilot")
    if not os.path.exists(binary_path):
        binary_path = download_binary()
        print(f"Downloaded binary to {binary_path}")
    else:
        print(f"Using existing binary at {binary_path}")
except Exception as e:
    print(f"Failed to download binary: {e}")
    print("You can manually download from: https://github.com/Dee66/CostPilot/releases")
    binary_path = None

setup(
    name="costpilot",
    version="1.0.0",
    description="Cost analysis and prediction for infrastructure as code",
    long_description="CostPilot is an AI-powered cost analysis engine that integrates into your CI/CD pipeline and gives you complete cost visibility before infrastructure changes go live.",
    long_description_content_type="text/plain",
    author="GuardSuite",
    author_email="info@guardsuite.com",
    url="https://github.com/Dee66/CostPilot",
    license="MIT",
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "Intended Audience :: System Administrators",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: System :: Systems Administration",
        "Topic :: Utilities",
    ],
    keywords="cost infrastructure terraform aws finops iac cloud",
    packages=[],
    package_data={
        "": ["bin/*"],
    },
    include_package_data=True,
    python_requires=">=3.8",
    entry_points={
        "console_scripts": [
            "costpilot=bin.costpilot:main",
        ],
    } if binary_path else {},
    data_files=[
        ("bin", [binary_path]) if binary_path else ("", []),
    ] if binary_path else [],
)