# SubCard

Subcard 是 Sub2Pool 的系统托盘参与者卡片。左键单击托盘图标切换置顶窗口的显示与隐藏；右键菜单用于切换极简模式、多选参与者、刷新、关闭卡片、控制开机启动和退出。

完整模式复用 Sub2Pool 的 `ParticipantCard.vue`、Tailwind CSS、daisyUI 主题和额度格式化规则；默认开启的极简模式使用更窄的窗口并隐藏次要信息，左右切换按钮仅在鼠标靠近卡片边缘时显示。

## 首版能力

- 使用 Sub2Pool API Key 调用 `/api/v1/participants` 和 `/api/v1/recommendations`
- 对首页待应用建议显示“应用建议”，并调用 `/api/v1/recommendations/{participant_id}/apply`
- API Key 保存到 Windows Credential Manager、macOS Keychain 或 Linux Secret Service
- 右键菜单直接列出全部参与者并支持多选
- 左右按钮或方向键切换已选参与者
- 极简模式默认开启，可从右键菜单关闭并恢复完整卡片
- 拖动卡片左边缘调整宽度并自动保存，重新启动后恢复上次宽度；窗口继续贴靠托盘所在的屏幕边缘
- 双击卡片、按 Esc 或使用右键菜单关闭窗口
- 打开时刷新，窗口打开期间每 60 秒刷新
- GitHub Actions 构建 Windows、macOS Intel/Apple Silicon 和 Linux 安装包

## 开发

需要 Node.js、pnpm、Rust 和对应平台的 Tauri 2 系统依赖。

```bash
pnpm install
pnpm tauri dev
```

`pnpm dev` 可在浏览器中使用开发数据预览卡片；访问 `http://localhost:1420/?settings=1` 可预览首次连接界面。

## 验证与构建

```bash
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

## 开源协议

Subcard 采用 [GNU Affero General Public License v3.0](LICENSE)（`AGPL-3.0-only`）发布。
