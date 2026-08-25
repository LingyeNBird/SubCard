# SubCard

Subcard 是 Sub2Pool 的系统托盘参与者卡片。左键单击托盘图标切换置顶窗口的显示与隐藏；右键菜单用于切换极简模式、多选参与者、刷新、关闭卡片、控制开机启动和退出。

完整模式复用 Sub2Pool 的 `ParticipantCard.vue`、Tailwind CSS、daisyUI 主题和额度格式化规则；默认开启的极简模式使用更窄的窗口并隐藏次要信息，左右切换按钮仅在鼠标靠近卡片边缘时显示。

### 桌面卡片演示
<img width="900" height="508" alt="image" src="https://github.com/user-attachments/assets/571ecbdc-1668-4d2e-a6db-aae46f515027" />


### 卡片内容演示
<img width="480" height="214" alt="PixPin_2026-08-25_20-59-49" src="https://github.com/user-attachments/assets/56c30a83-9075-42d1-b4c3-fd34dfa8d713" />
<img width="480" height="214" alt="PixPin_2026-08-25_20-59-41" src="https://github.com/user-attachments/assets/57a54ba2-ee82-48cd-ace6-8d665531c6d5" />

### 右键菜单演示

<img width="258" height="370" alt="PixPin_2026-08-25_21-04-50" src="https://github.com/user-attachments/assets/3ef06a0a-289f-4584-8dd8-f3fb2a8ccea6" />

### 连接设置演示

<img width="736" height="432" alt="image" src="https://github.com/user-attachments/assets/172ff958-f42b-4af4-9a7c-813561471a6d" />

### 应用建议演示


<img width="480" height="214" alt="PixPin_2026-08-25_20-59-34(1)" src="https://github.com/user-attachments/assets/ae8db06a-0190-4ec1-b5d2-a7c85f0ea08d" />
<img width="480" height="214" alt="PixPin_2026-08-25_20-59-59(1)" src="https://github.com/user-attachments/assets/bf07de7e-28e7-46dc-8f1f-a3a7a8ad33a5" />

## 能力

- 支持管理员和系统用户 API Key；系统用户只显示管理员授权的参与者，并且不能应用建议
- 管理员 Key 对首页待应用建议显示“应用建议”，并调用 `/api/v1/recommendations/{participant_id}/apply`
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

`pnpm dev` 可在浏览器中使用开发数据预览卡片；访问 `http://localhost:1420/?settings=1` 可预览首次连接界面，访问 `http://localhost:1420/?system-user=1` 可预览仅显示一个授权参与者且不能应用建议的系统用户界面。

## 验证与构建

```bash
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

## 发布 Release

Release 只能通过 GitHub Actions 中的“手动发布 Release”工作流手动触发。触发时填写不带 `v` 前缀的三段式版本标签，例如 `0.1.0`。工作流会校验项目版本、构建全部桌面平台、汇总上一个正式 Release 之后的所有提交，并在全部构建成功后发布 Release。

## 开源协议

Subcard 采用 [GNU Affero General Public License v3.0](LICENSE)（`AGPL-3.0-only`）发布。

## 原项目

[Sub2Pool拼车额度建议专业工具](https://github.com/LingyeNBird/Sub2Pool)

## 友链

[Linuxdo](https://linux.do/)
