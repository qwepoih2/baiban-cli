# npm 发布包完整指南：从注册到发布的每一步

> 记录将 baiban-cli 发布到 npm 过程中遇到的所有问题和解决方案，包括包名与作用域的选择、Token 认证、.npmrc 配置，以及常见报错排查。

## 前置准备

### 1. 注册 npm 账号

去 [npmjs.com](https://www.npmjs.com/signup) 注册一个账号，验证邮箱即可。

### 2. 确认登录状态

```bash
npm whoami
```

如果输出你的用户名，说明已经登录。否则需要登录（见下文）。

## 包名与作用域（Scope）

发布前第一个要做的决定：**用普通包名还是作用域包名？**

### 普通包

```json
{
  "name": "baiban-cli"
}
```

- 名字全局唯一，先到先得
- 安装命令简洁：`npm install baiban-cli`
- 适合开源项目、个人工具

### 作用域包

```json
{
  "name": "baiban-cli"
}
```

- `@kd` 是作用域，scope 内不会和别人撞名
- 安装命令：`npm install baiban-cli`
- 适合公司内部包、团队系列包
- 支持私有发布（付费）

### 怎么选？

| 场景 | 推荐 | 原因 |
|------|------|------|
| 开源工具 | 普通包 | 用户安装更方便，传播友好 |
| 公司内部 | 作用域包 | 不会撞名，支持私有，管理清晰 |
| 个人系列包 | 作用域包 | `@yourname/pkg1`、`@yourname/pkg2` 归属明确 |

### 检查包名是否可用

```bash
npm view 你的包名
```

如果返回 `404 Not Found`，说明包名可用。

## 配置 package.json

一个最小可发布的 `package.json`：

```json
{
  "name": "baiban-cli",
  "version": "0.1.0",
  "description": "Baiban CLI - 项目脚手架工具",
  "bin": {
    "baiban": "bin/baiban"
  },
  "scripts": {
    "postinstall": "node install.js"
  },
  "engines": {
    "node": ">=14"
  },
  "files": [
    "bin/",
    "install.js",
    "README.md"
  ],
  "repository": {
    "type": "git",
    "url": "https://github.com/qwepoih2/baiban-cli"
  },
  "keywords": ["cli", "scaffold", "template", "react"],
  "license": "MIT"
}
```

几个关键字段说明：

- **name**：包名，npm 上唯一标识
- **version**：版本号，遵循 [语义化版本](https://semver.org/)
- **bin**：CLI 工具必备，指定可执行文件路径
- **files**：发布时包含的文件列表，减小包体积
- **postinstall**：安装后自动执行的脚本

## 认证与发布

npm 发布强制要求 **两步验证（2FA）** 或 **Access Token**。有两种认证方式。

### 方式一：2FA + OTP（手动发布推荐）

1. 登录 [npmjs.com](https://www.npmjs.com/settings) → Account → Two-Factor Authentication → 开启
2. 登录并发布：

```bash
npm login
npm publish --otp=你的6位验证码
```

### 方式二：Access Token（自动化/CI 推荐）

#### 创建 Token

1. 登录 npmjs.com → Access Tokens → Generate New Token
2. 选择 **Granular Access Token**（细粒度访问令牌）

关键配置：

| 配置项 | 建议值 |
|--------|--------|
| 令牌名称 | 一个有意义的名字，如 `baiban` |
| 描述 | 可选，如 `发布 baiban-cli` |
| IP 地址范围 | 留空 |
| **包和作用域权限** | **Read and write，范围 All packages** |
| 组织 | 不配置 |
| 到期日 | 默认 30 天 |

> **最容易踩的坑**：很多人创建 Token 时忘了配置"包和作用域"的读写权限，导致发布时一直报 404。

#### 配置 .npmrc

创建项目根目录下的 `.npmrc`：

```
registry=https://registry.npmjs.org/
//registry.npmjs.org/:_authToken=npm_xxxxx你的token
```

> **注意格式**：`registry` 和 `_authToken` 必须分两行写。常见的错误写法是把它们拼在一行：
>
> ```
> # 错误写法
> registry=https://registry.npmjs.org/:_authToken=npm_xxx
> ```

#### 发布

```bash
npm publish
```

## 常见报错排查

### `ENOENT: Could not read package.json`

```
npm error enoent Could not read package.json: Error: ENOENT
```

**原因**：当前目录下没有 `package.json`。

**解法**：确认你在正确的目录执行命令。比如项目结构是 `项目根目录/npm/package.json`，需要先 `cd npm` 再发布。

### `403 Forbidden - Two-factor authentication required`

```
403 Forbidden - PUT https://registry.npmjs.org/xxx - Two-factor authentication or granular access token with bypass 2fa enabled is required
```

**原因**：npm 强制要求 2FA。

**解法**：开启 2FA 用 OTP 发布，或创建带 bypass 2FA 的 Access Token。

### `404 Not Found - PUT`

```
404 Not Found - PUT https://registry.npmjs.org/xxx
```

**原因**：Token 权限不足，最常见的是创建 Token 时没有配置"包和作用域"的读写权限。

**解法**：删掉旧 Token，重新创建时确保在"包和作用域"选择了 **Read and write**。

### `npm login` 打开浏览器无法输入

```bash
# 用 legacy 模式，在终端中输入用户名和密码
npm login --auth-type=legacy
```

Password 输入时**不会显示任何字符**（没有星号），看起来像卡住了，其实正在输入。直接粘贴 Token 后按回车即可。

### `npm login` 一直转圈

**原因**：网络问题或 npm registry 连接不稳定。

**解法**：跳过 `npm login`，直接在 `.npmrc` 中配置 Token（见上文），然后执行 `npm publish`。

## 版本管理

发布后更新版本号：

```bash
# 补丁版本 0.1.0 → 0.1.1
npm version patch

# 次要版本 0.1.0 → 0.2.0
npm version minor

# 主要版本 0.1.0 → 1.0.0
npm version major
```

更新后再次 `npm publish` 即可。

## 完整发布流程总结

```
1. 注册 npm 账号
2. 确定包名（普通包 vs 作用域包）
3. 检查包名可用性 → npm view 包名
4. 编写 package.json
5. 创建 Access Token（注意开启包的读写权限）
6. 配置 .npmrc（registry 和 authToken 分两行）
7. npm publish
```

就这么简单。祝发布顺利！
