#!/usr/bin/env node
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");
const os = require("os");

const VERSION = require("./package.json").version;
const REPO = "qwepoih2/baiban-cli";

// 检测平台
function getPlatformInfo() {
  const platform = os.platform();
  const arch = os.arch();

  const platformMap = {
    "darwin arm64":  "baiban-darwin-arm64",
    "darwin x64":    "baiban-darwin-x64",
    "linux x64":     "baiban-linux-x64",
    "linux arm64":   "baiban-linux-arm64",
    "win32 x64":     "baiban-win32-x64.exe",
    "win32 arm64":   "baiban-win32-arm64.exe",
  };

  const key = `${platform} ${arch}`;
  const binaryName = platformMap[key];

  if (!binaryName) {
    throw new Error(`不支持的平台: ${platform} ${arch}`);
  }

  return { binaryName, isWindows: platform === "win32" };
}

// 下载文件
function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        download(res.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      if (res.statusCode !== 200) {
        reject(new Error(`下载失败: HTTP ${res.statusCode}`));
        return;
      }
      res.pipe(file);
      file.on("finish", () => {
        file.close(resolve);
      });
    }).on("error", reject);
  });
}

async function install() {
  const { binaryName, isWindows } = getPlatformInfo();
  const binDir = path.join(__dirname, "bin");
  const binaryPath = path.join(binDir, isWindows ? "baiban.exe" : "baiban-bin");

  // GitHub Release 下载地址
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${binaryName}`;

  console.log(`正在下载 baiban v${VERSION} (${binaryName})...`);

  try {
    fs.mkdirSync(binDir, { recursive: true });
    await download(url, binaryPath);

    // 设置可执行权限（非 Windows）
    if (!isWindows) {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log("baiban 安装成功！");
  } catch (err) {
    console.error("安装失败:", err.message);
    process.exit(1);
  }
}

install();
