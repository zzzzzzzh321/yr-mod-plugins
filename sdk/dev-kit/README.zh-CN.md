# VERA20K 插件独立开发包

本开发包用于在没有 VERA20K 引擎源码的环境中，修改、测试并编译 `vera20k-ares.wasm`。编译只依赖插件源码、SDK 的类型与接口定义，以及 Rust 工具链。

本说明维护在独立 `plugin-dev` 工程的 `sdk/dev-kit/README.zh-CN.md`，也可作为导出开发包的根 README。下文所有路径和命令均以**插件开发工程或导出开发包的根目录**为起点。

## 开发包内容

```text
开发包根目录/
├── Cargo.toml                   # 仅 SDK 与插件的独立 workspace
├── Cargo.lock                   # 独立开发包的依赖版本
├── LICENSE-GPL
├── sdk/
│   ├── api/                     # 请求、结果、公共数据类型与计时器
│   ├── wit/                     # 插件与引擎之间的接口定义
│   ├── package/                 # 通用组件打包工具源码
│   └── dev-kit/                 # 本说明与 workspace 模板
└── plugins/
    └── vera20k-ares/            # 可修改的功能插件源码与 INI 示例
```

根 workspace 只有 `vera-mod-api`、`vera-mod-pack` 和 `vera20k-ares` 三个 crate，默认构建成员为插件。其他第三方依赖由 Cargo 按锁文件获取。这个工程不包含引擎 crate、Wasmtime 运行时或游戏资源，也不需要 Python 和引擎专用的 `tools.cargo_run`。

后续插件源码和打包工具以本工程为维护位置。`sdk/api`、`sdk/wit` 是从引擎权威接口取得的固定版本开发依赖，来源见 `sdk/source.json`；升级接口时明确同步，日常插件编译只访问本地快照。

SDK 中的 API 和 WIT 类似 C++ 插件开发中的公共头文件与调用契约：它们说明双方交换哪些数据、可以调用哪些接口。具体的 Host 服务由运行插件时的引擎提供，编译插件时无需链接引擎源码或可执行文件。

## 准备环境

安装支持 Rust 2024 edition 的 Rust 工具链，确保 Cargo 和当前平台所需的链接器可用。然后安装插件编译目标：

```sh
rustup target add wasm32-unknown-unknown
```

首次构建通常需要联网下载 Cargo 依赖；已缓存的依赖可以复用。保留开发包附带的 `Cargo.lock`，以使用同一组依赖版本。

## 修改与测试

功能代码位于 `plugins/vera20k-ares/src/features/`，仍按防御、武器等领域组织。公共宿主调用集中在 `src/support/host.rs`，组件入口和功能组合位于 `src/component.rs`。

从开发包根目录运行：

```sh
cargo test --locked -p vera-mod-api -p vera20k-ares -p vera-mod-pack --lib
```

这些测试检查插件决策与打包行为。它们可以在没有 `vera20k.exe` 和原版游戏资源的环境中运行；实际游戏效果、引擎接入、存读档和联机行为仍需放入兼容引擎中验收。

## 编译插件

```sh
cargo build --locked --release --target wasm32-unknown-unknown -p vera20k-ares
```

原始输出为：

```text
target/wasm32-unknown-unknown/release/vera20k_ares.wasm
```

这是包含 WIT 元数据的 core Wasm，接下来需要封装为引擎加载的组件。此处使用 `wasm32-unknown-unknown`，保持当前 SDK 所要求的导入接口。

## 生成发布文件

先在开发包根目录创建 `dist` 文件夹，再执行以下命令。输出子目录 `release-new` 必须尚不存在；下一次发布使用另一个新子目录。

```sh
cargo run --locked --release -p vera-mod-pack -- target/wasm32-unknown-unknown/release/vera20k_ares.wasm dist/release-new --program vera20k-ares.wasm
```

Cargo 会编译本机使用的打包工具并执行，产物为：

```text
dist/release-new/
├── vera20k-ares.wasm
└── vera-mod.json
```

声明中的文件名与内容摘要由打包工具生成。将这两个文件一起放入作品资源系统，再用作品实际加载的 INI 选择功能。作品的其他美术、音频、地图和规则资源沿用原有分发流程。

如果设置了 `CARGO_TARGET_DIR`，原始 Wasm 的位置随之改变；打包命令中的输入路径也应使用该目标目录。源码修改和编译过程不会自动改变作品当前对局中的组件。

## SDK 与引擎版本

当前接口为 SDK API v1，WIT 名称为 `vera:mod-sdk@0.1.0`。参考插件保留内部标识 `vera.reference-gameplay`、组件版本 `0.6.0`、配置版本 `5` 和状态版本 `1`。发布文件名仍为 `vera20k-ares.wasm`。0.6.0 需要能接收 `use_weapon_rof` 和可选 `input` 类型策略的宿主；旧宿主会拒绝未知字段，SDK 快照来源见 `sdk/source.json`。

修改现有接口能够表达的算法，可以只重编插件。需要新的引擎能力时，应先扩展 SDK 契约和引擎接入点，再使用匹配的 SDK 构建插件；独立编译并不表示所有未来 SDK 与引擎版本自动兼容。

引擎加载时会检查组件声明、摘要、接口及允许的导入。读档仍要求对应的组件内容，因此更新插件时保留旧版本文件，供旧存档使用。开发包沿用 GPL 授权，见 `LICENSE-GPL`。

## 维护者如何组装开发包

以本插件开发工程为来源，在新的输出目录保持上述相对布局，复制 `sdk/api`、`sdk/wit`、`sdk/package`、`sdk/dev-kit`、`sdk/source.json`、`plugins/vera20k-ares` 和许可证。复制根 `Cargo.toml` 与 `Cargo.lock`，将本说明复制为导出包的根 README。模板 `sdk/dev-kit/Cargo.toml.template` 也可用于建立新的同结构工程。

本工程的锁文件已经只包含 SDK、插件与打包工具的依赖。在依赖已缓存的维护环境中，可执行以下命令检查导出工程仍能按该版本解析：

```sh
cargo metadata --locked --offline --format-version 1
```

检查依赖图只包含开发包成员及其依赖，再使用前述带 `--locked` 的测试、构建和打包命令验证。不要将引擎 workspace 或引擎锁文件作为本开发包的根配置。

源码以 `plugin-dev` 为准，导出包是交付快照。一般作者直接使用包内锁文件；只有维护者明确升级依赖时才重新解析版本。

高于主/副武器的 NoAmmoWeapon 索引还需要能完整保存建筑延迟武器索引的配套宿主。SDK 消息与 WIT 未改变，不代表旧宿主内的武器处理缺陷会自动修复；按 source.json 对应引擎或配套验收包运行。
