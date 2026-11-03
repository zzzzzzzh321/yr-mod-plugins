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

声明中的文件名与内容摘要由打包工具生成。将这两个文件一起放入作品资源系统，再用作品实际加载的INI配置功能。0.7.0稳定交付的新对局默认启用`AresAmmo`、`AresReload`、`AresNoAmmoWeapon`、`AresManualControl`。当前0.14.0开发分支还默认启用`AresKeepAlive`、`AresAttachEffect`、`AresArmor`、`AresProduction`、`AresEMP`、`AresSpotlight`、`AresTypeConversion`和`AresFoundation`，迁移引擎已有的存活判定、附加效果及其隐形/属性依赖、自定义护甲和建筑占地等支持。可以直接使用原Ares标签；`[VeraFeatures]`中对应项设为`no`可分别关闭。标签自身的默认值不变，四个手动限制仍默认`no`。Phobos护盾和自定义过载仍须显式`Shield=yes`与`Overload=yes`，过载依赖护盾。作品的其他美术、音频、地图和规则资源沿用原有分发流程。

如果设置了 `CARGO_TARGET_DIR`，原始 Wasm 的位置随之改变；打包命令中的输入路径也应使用该目标目录。源码修改和编译过程不会自动改变作品当前对局中的组件。

## SDK 与引擎版本

接口编号仍为 SDK API v1，WIT 名称为 `vera:mod-sdk@0.1.0`。参考插件保留内部标识`vera.reference-gameplay`和发布文件名`vera20k-ares.wasm`。稳定交付0.7.0使用配置版本5、状态版本1；当前0.14.0开发分支使用配置版本14、状态版本1；探照灯、类型转换和建筑占地增量已完成集中验收，整体迁移尚未完成，也未替换保留交付。

开发分支新增`match_rules`、`effect_resource`、`effects`、`armor_resource`、`production_resource`、`configure_boundary`和`emp_resource`可选能力，以及类型隐形许可等消息字段。宿主须提供物理Rules读取点的注册类型查询、效果实例事务、原数学/动画/隐形所有者和相应保存恢复接入；只认识原有回调的宿主无法运行这版组件。组件的护盾字节与宿主效果列表分别保存，效果列表只有一个所有者。宿主接收静态策略时建立派生缓存，每次效果更新只提交经校验的有序动作。护甲增量已通过关联回归及真实资源加载，整体迁移仍在进行。SDK快照的配套引擎来源提交见`sdk/source.json`。

修改现有接口能够表达的算法，可以只重编插件。需要新的引擎能力时，应先扩展 SDK 契约和引擎接入点，再使用匹配的 SDK 构建插件；独立编译并不表示所有未来 SDK 与引擎版本自动兼容。

引擎加载时会检查组件声明、摘要、接口及允许的导入。新默认只用于新对局的初始配置，旧的非空冻结配置不补入新默认；旧存档继续使用保存的原组件字节与配置，不重读当前INI。当前作品资源还必须提供与存档摘要一致的组件，因此更新插件时保留旧组件及声明，供旧存档使用。开发包沿用 GPL 授权，见 `LICENSE-GPL`。

## 维护者如何组装开发包

以本插件开发工程为来源，在新的输出目录保持上述相对布局，复制 `sdk/api`、`sdk/wit`、`sdk/package`、`sdk/dev-kit`、`sdk/source.json`、`plugins/vera20k-ares` 和许可证。复制根 `Cargo.toml` 与 `Cargo.lock`，将本说明复制为导出包的根 README。模板 `sdk/dev-kit/Cargo.toml.template` 也可用于建立新的同结构工程。

本工程的锁文件已经只包含 SDK、插件与打包工具的依赖。在依赖已缓存的维护环境中，可执行以下命令检查导出工程仍能按该版本解析：

```sh
cargo metadata --locked --offline --format-version 1
```

检查依赖图只包含开发包成员及其依赖，再使用前述带 `--locked` 的测试、构建和打包命令验证。不要将引擎 workspace 或引擎锁文件作为本开发包的根配置。

源码以 `plugin-dev` 为准，导出包是交付快照。一般作者直接使用包内锁文件；只有维护者明确升级依赖时才重新解析版本。

高于主/副武器的 NoAmmoWeapon 索引还需要能完整保存建筑延迟武器索引的配套宿主。API编号或WIT包名相同，不代表旧宿主具有新回调、消息字段或处理修复；按来源提交与配套验收包核对。新增宿主保存字段会改变完整存档布局，保留旧引擎和旧组件供旧存档使用，不能把护盾状态版本仍为1理解成完整存档自动兼容。

生产扩展在 `features/production` 按组、工厂资格与公共目录索引组织；没有第二份 House 计数、工厂队列或永久计划。配置边界由宿主真实流程提供：普通场景清空，类型重建保留组件自身的全局控制，读取存档直接采用保存配置。跨场景种子必须匹配完全相同的组件内容与清单，更新组件后应开启新对局。0.10.0 普通引擎新增国家来源与计划字段，完整存档布局随之变化；旧存档保留相应旧引擎和组件，不自动转换。生产增量已通过关联回归、旧实现对照、正式读档和七种真实资源加载检查；整体迁移尚未完成。

EMP 扩展位于 `features/units/emp`。新配置默认启用 `AresEMP`，历史冻结配置缺少该开关时保持关闭；`[VeraFeatures] AresEMP=no` 明确停用。插件只发布静态声明，计数、停用状态和动画引用仍只有原宿主所有者。任务名称、阈值和注册动画由共享 Host 调用原读取器，类型动画的继承／清空／指定三态不会合并。TypeReset 保留 EMP 全局任务配置；普通场景清空，读档使用保存配置。EMP 增量已通过关联行为、旧实现对照、正式读档和七种真实资源加载检查。完整存档新增触发器冻结策略，旧存档需保留对应旧引擎和组件；整体迁移继续进行。

探照灯通过现有 `type_resource.spotlight` 发布静态声明，没有新增逐帧回调或灯光状态。新对局默认启用 `AresSpotlight`；关闭时恢复原版建筑灯规则。两套开关沿各自的物理读取时点累积，不能从最终合并文本重新猜测。原版 General 参数继续由宿主保留，颜色与几何交给原绘制链。本增量已通过关联行为、旧实现对照、应用资源加载、现有 GPU 像素对照和七种真实资源加载检查；原有绘制残差继续保留。

类型转换通过现有 `type_resource.conversion` 与 `match_rules.script_type_conversion` 发布冻结声明。`AresTypeConversion` 默认启用，关闭时恢复原版 Script68 不执行行为。`Convert.Script` 使用真实读取点的共同 Techno 顺序，保留类别和编号；`TurretROT` 保留完整有符号输入，只在转换时交原 Facing 所有者处理。单位状态、队伍游标和运动控制器仍由原引擎拥有。本增量已通过关联行为、旧实现对照、正式资源恢复和七种真实资源加载检查。普通引擎接纳停用矿车枚举分支后会改变完整存档布局；旧存档保留对应旧引擎和组件。

建筑占地使用独立的 `foundation_resource` 声明，源码在 `features/buildings/foundation.rs`。插件按真实 Building 读取点保存原版占地编号和第一次 `Foundation=Custom` 的固定 ART section 引用，关闭期间也保留这段历史；`AresFoundation=no` 只改变最终选择。插件不保存或逐格读取几何数组。宿主校验来源、内容散列与 section 后，由原 Foundation 模块生成完整占地和轮廓，摆放、占格、寻路与存档沿用原有所有者。读档复用保存的配置和匹配的固定 ART，不重新执行配置。普通引擎通过最终绑定检查自定义几何；旧内置 Ares 编译模式中，原始非法 Custom 仍会在更早的规则校验处失败，即使组件随后关闭该功能。

本占地增量已通过真实组件的空间与存档测试、关联回归、旧实现对照、正式客户端及现有摆放 GPU 检查、七种真实资源加载模式和上一版组件加载检查。这不表示所有自定义占地绘制路径都已与原生逐像素对照。普通构建新增 Custom 类型后，完整存档布局和配置摘要会变化；旧存档保留对应旧引擎与组件。
