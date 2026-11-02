# 插件开发工程约定

本目录是 VERA20K 外置功能插件的主要源码维护位置。用户明确指令优先。

## 职责与所有权

- 功能实现维护在 `plugins/vera20k-ares/src/features`，按领域和完整机制组织。
- 所有功能共用 `src/support/host.rs`，组件入口在 `src/component.rs`。
- 插件开发、构建与发布工具维护在 `sdk/package`；开发指南在 `sdk/dev-kit`。
- `sdk/api`、`sdk/wit` 是固定版本的 SDK 开发依赖快照，来源见 `sdk/source.json`。接口的权威源码仍在相邻 `../engine/sdk/api` 和 `../engine/sdk/wit`，不要把快照作为另一份独立接口实现维护。
- 需要新宿主能力时，先在引擎维护接口和通用接入，再明确同步 SDK 快照并检查版本、兼容性和相关测试。现有接口范围内的玩法修改只在本工程进行。
- `../engine` 保留引擎、插件加载器、运行时、协议和宿主接入；不要把插件玩法或开发工具重新放回引擎仓库。
- `../exports` 是历史交付快照，不作为新的源码修改位置。

## 构建与验证

本工程独立于引擎 workspace，直接使用标准 Cargo，不需要引擎的 Python 构建器、引擎源码或游戏资源。

```sh
cargo test --locked -p vera-mod-api -p vera-mod-pack -p vera20k-ares --lib
cargo build --locked --release --target wasm32-unknown-unknown -p vera20k-ares
```

本机已建立的插件编译缓存为 `E:/VERA-Community/work/sdk-standalone-target`，需要复用时设置 `CARGO_TARGET_DIR`；否则 Cargo 使用本工程的 `target`。设置外部缓存后，相应调整打包命令的输入路径。Windows、Linux、macOS 保持同一源码结构。

一次只运行一组 Cargo，不与其他任务争抢或终止编译进程。执行构建期间不修改本工程源码。按变更选择针对性测试，玩法修改要验证实际组件与正式引擎路径，不能只验证配置字段。

发布组件仍为 `vera20k-ares.wasm` 与 `vera-mod.json`。使用打包器生成匹配的文件名和摘要。保存旧组件供旧存档使用；不要在运行中的对局替换组件。

## 版本管理与资料

本工程使用独立本地 Git 仓库；在功能分支工作，提交已验证的修改。未经用户授权不推送、上传或公开。保留原代码许可证、贡献者说明和原始行为引用。

编译产物写入 `target` 或外部缓存，发布文件写入 `dist`。日志、验证报告和临时研究资料放入 `../work`。自有测试地图和 INI 样例可以随源码维护；不要提交原版或第三方游戏素材、编译缓存或测试运行记录。
