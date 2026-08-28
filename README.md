注意！！！
项目需要导入1.12.2的资源，手动导入十分复杂，推荐使用自动导入脚本，但是自动导入脚本不一定能成功运行，这主要取决于你的环境与目标资源文件，可以使用本机的1.12.2资源，但推荐使用官方重写参照的MCP-1.12.2-main.zip，这个自动导入资源脚本成功运行率更高https://1850640083.share.123pan.cn/123pan/RvAxvd-NIOgA

# Minecraft Java Edition 1.12.2 — Rust Semantic Port

> 使用 Rust 对 Minecraft Java Edition 1.12.2 客户端进行语义级移植，并提供 Vulkan 与 OpenGL 双渲染后端。

当前公开基线：**0.127.0**<br>
当前重点平台：**Windows 10/11 x64**<br>
协议目标：**Minecraft Java Edition 1.12.2 / Protocol 340**

---

## 项目简介

本项目的目标不是制作一个“外观类似 Minecraft”的独立仿制游戏，而是在 Rust 中尽可能忠实地重建 Minecraft Java Edition 1.12.2 客户端的行为、状态、调用流程、网络协议、GUI、模型、动画和操作体验。

原版 Minecraft 1.12.2 与对应 MCP 代码结构是行为基准。Rust 与 Java 的语言机制不同，因此资源所有权、并发、内存管理和图形 API 提交方式采用 Rust 等价实现，但可观察行为应尽量保持与 1.12.2 一致。

项目当前已经同时具备远程多人客户端与正在迁移中的单人 IntegratedServer 路径。v0.127.0 已实现 Flat 世界的真实 LocalChannel/IntegratedServer 进入链、服务器权威区块修改与 Anvil/playerdata 持久化，并已把 Default/Default 1.1/Large Biomes/Amplified/Customized 的主世界生成推进到 MCP 派生的 GenLayer、BiomeProvider、Noise、Biome surface、洞穴和峡谷主干。这仍不等于整个 Minecraft 1.12.2 已无差异完成：结构生成、population/decorator、Nether/End、完整 Entity/TileEntity 生命周期以及部分复杂交互仍在继续迁移。

## 核心原则

- **MCP 1.12.2 优先**：能从原版源码确认的行为按原版实现，不凭印象重新设计。
- **保持职责边界**：尽量保留 `Minecraft`、`RenderGlobal`、`RenderChunk`、`RenderManager`、`TileEntityRendererDispatcher`、GUI、容器和协议类的职责关系。
- **增量移植**：在现有实现上补齐差异，不用占位代码、静态截图、假数据或统一模型冒充真实功能。
- **双后端同语义**：Vulkan 与 OpenGL 使用同一套 MCP 派生的世界、实体和 GUI 状态，后端只负责原生资源和提交。
- **优化不改变行为**：缓存、常驻显存、批处理和并行构建不得改变可见性、透明顺序、实体顺序、动画或 OptiFine 程序边界。

## v0.127.0 当前进展

- Flat 单人世界通过真实 `IntegratedServer -> LocalChannel -> Login/Play -> WorldClient` 链进入，不使用客户端静态地形替代服务器。
- 方块修改开始由 `PlayerInteractionManager / WorldServer / Chunk` 权威处理，并进入 Anvil 异步保存；玩家位置、背包、当前槽位等写入 `playerdata/<UUID>.dat`。
- Default、Default 1.1、Large Biomes、Amplified、Customized 已接入真实 `IntCache / GenLayer / BiomeProvider / NoiseGenerator / ChunkGeneratorOverworld` 基础地形，并包含 biome surface、洞穴和峡谷。
- Vulkan/OpenGL 继续共享 MCP 派生场景状态；OpenGL 已包含 resident-span 局部 `BufferSubData` 更新等性能路径。
- 尚未完成的主项包括 Overworld structures 与 population/decorator、Nether/End generator、完整 TileEntity/复杂多方块放置、完整服务端实体生命周期与更多单人服务器行为。

## 主要特色

### Minecraft 1.12.2 语义结构

源码目录尽量镜像 MCP 包路径，包括：

- `net.minecraft.client`
- `net.minecraft.entity`
- `net.minecraft.block`
- `net.minecraft.item`
- `net.minecraft.network`
- `net.minecraft.world`
- `net.optifine`

项目包含协议 340 的登录、加密和多人游戏数据路径，以及 GUI、HUD、物品栏、容器、资源包、声音、玩家皮肤/披风、方块状态、实体渲染、粒子和维度相关实现。当前版本还包含内置账号管理器、Microsoft 浏览器 OAuth、Token/Offline 会话切换和远程玩家名称标签。各系统的完成程度并不完全相同，公开发布时不应将本项目描述为原版客户端的无差异替代品。

### Vulkan 渲染后端

Vulkan 路径使用 Vulkan 1.1，主要技术包括：

- 原版 `RenderChunk`、`CompiledChunk` 与 `VisGraph` 可见性结构；
- `SOLID`、`CUTOUT_MIPPED`、`CUTOUT`、`TRANSLUCENT` 四个方块渲染层；
- 共享区块顶点/索引显存池；
- 设备本地常驻区块网格；
- 多绘制间接命令；
- 有界区块编译和上传队列；
- 原版透明四边形中心距离排序与运行时重新排序；
- 动态实体、方块实体和静态悬挂实体独立 GPU 流；
- Vulkan 原生 GUI、全景主菜单和异步纹理上传；
- 帧槽 Fence 驱动的资源延迟回收。

### OpenGL 渲染后端

OpenGL 路径创建 OpenGL 3.3 Compatibility Profile，主要技术包括：

- 与 Vulkan 共用的 MCP 场景构建结果；
- `RenderRegion` 驻留和 `MultiDrawElements`；
- 精确透明索引区间更新；
- 原版实体与方块实体程序边界；
- OptiFine 1.12.2 风格的 G-buffer、composite、final 与 shadow 程序路径；
- Shader Options、维度目录、include 展开和光影包配置。

**OptiFine 光影仅在 OpenGL 后端启用。** Vulkan 后端不会直接运行传统 OptiFine GLSL 光影包。

### 内置账号管理器

主菜单中的 `Accounts` 页面提供本地账号列表和会话切换，当前支持：

- Microsoft 浏览器 OAuth 登录；
- 已保存 Microsoft 账号的访问令牌登录和刷新令牌续期；
- Minecraft Access Token 登录；
- `M.C` 刷新令牌登录；
- Offline 用户名会话；
- 账号排序、删除、双击登录、头像显示和当前账号高亮；
- 使用当前 Minecraft Access Token 上传 Classic 或 Slim 皮肤。

认证成功后会替换客户端真实 `Session`，并继续使用 1.12.2 的 `NetHandlerLoginClient → joinServer` 认证链，不是只修改界面用户名。

账号数据保存在：

```text
config/account.json
```

为保持与参考账号管理器兼容，该文件包含明文刷新令牌和 Minecraft Access Token。仓库的 `.gitignore` 已忽略整个 `config/`，但提交前仍必须检查 Git 变更列表，确保没有通过强制添加、旧提交或其他路径泄露账号凭据。

### 远程玩家名称标签

玩家名称标签按 Minecraft 1.12.2 的 `RenderLivingBase`、`RenderPlayer`、`ScorePlayerTeam` 与 `Scoreboard` 行为实现，包括：

- 普通玩家 64 格、潜行玩家 32 格显示距离；
- 队伍前缀、后缀、颜色与四种名称可见规则；
- 友军隐身可见和旁观者相关判断；
- 普通名牌的穿墙暗色层与深度测试亮色层；
- 潜行名牌的遮挡和深度写入规则；
- 10 格内显示记分板显示槽 2 的分数与目标名称；
- 第三人称不显示本地玩家自己的名称。

### 资源系统

项目不会在仓库中捆绑客户端运行所需的完整 Mojang 资源集合。源码中仅保留构建或界面所需的少量内嵌元数据与默认图标；完整纹理、声音、字体、模型和语言资源仍由维护者或用户从合法本地 Minecraft 安装和 MCP 资源中导入。

资源导入完成后位于：

```text
runtime/assets/
└─ minecraft/
   ├─ blockstates/
   ├─ lang/
   ├─ models/
   ├─ sounds/
   ├─ textures/
   ├─ optifine/
   └─ mcpatcher/
```

## 仓库结构

```text
.
├─ .github/workflows/ci.yml       # GitHub Actions 格式和 Release 编译检查
├─ src/
│  ├─ net/minecraft/              # MCP 1.12.2 语义移植主体
│  ├─ net/optifine/               # OptiFine 兼容结构
│  ├─ compat/                     # Java 语义的 Rust 等价实现
│  ├─ launcher/                   # 启动、资源根目录和后端选择
│  ├─ renderer/                   # Vulkan/OpenGL 公共后端接口
│  ├─ vulkan/                     # Vulkan 窗口、GUI、世界管线和着色器
│  ├─ opengl/                     # OpenGL 世界、GUI 与 OptiFine 光影运行时
│  └─ bin/                        # 独立资源验证工具
├─ tools/
│  ├─ one_click_import_assets.py  # 事务式资源导入器
│  └─ asset-validator/            # 隔离的 Rust 资源验证器
├─ resourcepacks/                 # Java 1.12.2 材质包目录
├─ shaderpacks/                   # OptiFine 光影包目录，仅 OpenGL
├─ Import-Assets-OneClick.cmd     # Windows 一键资源导入入口
├─ Build-And-Run.cmd              # Windows Release 构建和运行入口
├─ Cargo.toml
├─ build.rs                       # 构建时编译 Vulkan GLSL 为 SPIR-V
├─ rust-toolchain.toml
├─ LICENSE
└─ README.md
```

## 环境要求

### 已验证平台

- Windows 10 或 Windows 11，64 位；
- 支持 Vulkan 1.1 的显卡及较新的显卡驱动；
- 支持 OpenGL 3.3 Compatibility Profile 的显卡及驱动。

其他操作系统可能可以编译部分代码，但当前公开基线主要在 Windows 上开发和验证。

### 构建工具

建议安装：

1. [Rust 工具链](https://rustup.rs/)；
2. Visual Studio 2022 Build Tools；
3. “使用 C++ 的桌面开发”工作负载；
4. Windows 10/11 SDK；
5. [Vulkan SDK](https://vulkan.lunarg.com/sdk/home)；
6. CMake；
7. Python 3.9 或更高版本。

仓库中的 `rust-toolchain.toml` 会选择 stable Rust，并安装 `rustfmt` 与 `clippy`。`Cargo.toml` 声明的最低 Rust 版本为 1.77，但公开构建建议直接使用当前 stable 工具链。

`shaderc` 构建依赖会在编译期间构建着色器编译组件，因此第一次编译通常明显慢于后续增量编译。

## 快速开始

### 1. 获取仓库

```bat
git clone <repository-url>
cd Minecraft-1.12.2-Rust
```

也可以直接下载 GitHub ZIP 并解压到新的空目录。

### 2. 准备资源来源

需要同时具备：

- 一份由正版启动器下载的 Minecraft 1.12 系列本地资源；
- 一份包含 `src/assets` 的 MCP 1.12.2 目录或 ZIP。

默认情况下，一键脚本会自动寻找：

```text
%APPDATA%\.minecraft\assets\indexes
%APPDATA%\.minecraft\assets\objects
```

并在仓库根目录、仓库父目录和当前工作目录中寻找名称类似以下形式的文件：

```text
MCP-1.12.2-main.zip
MCP-1.12.2-*.zip
*MCP*1.12.2*.zip
```

推荐布局：

```text
Minecraft-1.12.2-Rust/
├─ MCP-1.12.2-main.zip
├─ Import-Assets-OneClick.cmd
├─ Cargo.toml
└─ src/
```

MCP ZIP 是否适合公开再分发取决于其来源和许可证。仓库维护者必须自行确认有权上传其中的代码、库和资源。本项目的许可证不会授予这些第三方材料的分发权。

### 3. 一键导入资源

双击：

```text
Import-Assets-OneClick.cmd
```

导入成功后会生成：

```text
runtime/assets/
runtime/asset-import-report.json
```

### 4. 构建并运行

双击：

```text
Build-And-Run.cmd
```

或者在终端执行：

```bat
cargo run --release --bin mc112-client -- run
```

## 一键资源导入脚本

### 脚本入口

`Import-Assets-OneClick.cmd` 是 Windows 包装脚本。它负责：

1. 将工作目录切换到仓库根目录；
2. 依次寻找 `py.exe`、`python.exe` 或 `python3.exe`；
3. 启用 UTF-8 控制台和 Python UTF-8 模式；
4. 调用 `tools/one_click_import_assets.py`；
5. 返回 Python 导入器的真实退出码。

### Python 导入器原理

资源导入器采用事务式流程：

1. 查找合法本地 `.minecraft` 目录；
2. 选择 `assets/indexes/1.12.json`，找不到时再尝试相邻的 1.12 系列索引；
3. 根据资产索引中的 SHA-1，将 `assets/objects` 的哈希对象还原成逻辑资源路径；
4. 校验每个官方对象的 SHA-1；
5. 查找 MCP 目录或 ZIP；
6. 提取并覆盖合并 MCP 的 `src/assets`；
7. 检查 GUI、字体、语言、声音和 OptiFine/MCPatcher 资源覆盖；
8. 仅在暂存目录完全通过验证后，原子替换 `runtime/assets`；
9. 如果本机存在 Cargo，再运行隔离的 Rust 资源验证器；
10. 写入 JSON 导入报告。

这一流程避免在导入中途失败时留下半套资源。旧的 `runtime/assets` 只有在新资源验证通过后才会被替换。

### 手动指定路径

```bat
py -3 -X utf8 tools\one_click_import_assets.py ^
  --project-root . ^
  --minecraft-dir "%APPDATA%\.minecraft" ^
  --mcp ".\MCP-1.12.2-main.zip"
```

非交互模式：

```bat
py -3 -X utf8 tools\one_click_import_assets.py ^
  --project-root . ^
  --minecraft-dir "%APPDATA%\.minecraft" ^
  --mcp ".\MCP-1.12.2-main.zip" ^
  --non-interactive
```

可用参数：

```text
--minecraft-dir PATH       指定 .minecraft 目录
--mcp PATH                 指定 MCP 文件夹、src/assets 或 ZIP
--index NAME               指定资产索引名称，默认 1.12
--destination PATH         指定输出目录，默认 runtime/assets
--non-interactive          禁止交互式询问
--skip-cargo-validation    跳过隔离的 Rust 资源验证器
--no-hash-check            跳过官方对象 SHA-1 校验，不建议使用
```

## 完全手动导入资源

官方 Minecraft 资产以哈希对象形式存储，不能直接把 `.minecraft/assets/objects` 整个复制到 `runtime/assets`。

完全手工导入需要执行以下逻辑：

1. 打开 `.minecraft/assets/indexes/1.12.json`；
2. 遍历 `objects` 中的每个逻辑资源名和 SHA-1；
3. 对于 SHA-1 `abcdef...`，读取：

   ```text
   .minecraft/assets/objects/ab/abcdef...
   ```

4. 将该文件复制为：

   ```text
   runtime/assets/<索引中的逻辑资源名>
   ```

5. 解压 MCP ZIP；
6. 将 MCP 的 `src/assets` 内容覆盖合并到 `runtime/assets`；
7. 确认至少存在：

   ```text
   runtime/assets/minecraft/lang/en_us.lang
   runtime/assets/minecraft/textures/gui/title/minecraft.png
   runtime/assets/minecraft/textures/gui/widgets.png
   runtime/assets/minecraft/textures/font/ascii.png
   runtime/assets/minecraft/sounds.json
   runtime/assets/minecraft/sounds/*.ogg
   runtime/assets/minecraft/optifine/ 或 mcpatcher/
   ```

8. 执行资源验证：

   ```bat
   cargo run --release --bin validate-assets -- --path runtime/assets
   ```

除非需要研究资源索引格式，否则推荐使用一键脚本。它执行的是同一套逻辑，但带有哈希校验、暂存目录和失败回滚。

## 构建方法

### Release 构建

```bat
cargo build --release --bin mc112-client
```

生成文件：

```text
target/release/mc112-client.exe
```

运行：

```bat
target\release\mc112-client.exe run
```

### `Build-And-Run.cmd` 的作用

该脚本只执行必要步骤：

1. 检查 Cargo 是否可用；
2. 检查 `runtime/assets` 是否已经导入；
3. 执行优化的 Release 构建；
4. 直接启动生成的 `mc112-client.exe`；
5. 将脚本收到的参数继续传给 `run` 子命令。

例如：

```bat
Build-And-Run.cmd --width 1280 --height 720 --fullscreen
```

### 代码检查

```bat
cargo fmt --all -- --check
cargo check --release --all-targets
cargo test --release
```

GitHub Actions 只执行格式和 Release 编译检查，不运行需要真实资源、显示器或 GPU 的图形测试。

## 启动参数

默认运行：

```bat
cargo run --release
```

等价于：

```bat
cargo run --release -- run --assets runtime/assets --width 854 --height 480
```

常用参数：

```text
--assets PATH              运行时资源根目录
--width NUMBER             初始窗口宽度
--height NUMBER            初始窗口高度
--fullscreen               全屏启动
--username NAME            会话用户名
--uuid UUID                玩家 UUID 或会话标识
--accessToken TOKEN        外部启动器提供的访问令牌
--userType TYPE            会话类型，默认 legacy
```

示例：

```bat
cargo run --release -- run ^
  --assets runtime/assets ^
  --width 1280 ^
  --height 720 ^
  --username Player
```

仓库已经包含内置账号管理器与 Microsoft 浏览器 OAuth。也可以继续通过启动参数从合法外部启动器传入会话信息。无论使用哪种方式，都不要把访问令牌、刷新令牌、`config/account.json`、截图中的账号信息或认证日志提交到 GitHub。

其他工具命令：

```bat
cargo run --release -- version
cargo run --release -- probe-vulkan
cargo run --release -- validate-assets --path runtime/assets
cargo run --release -- render-main-menu-preview --assets runtime/assets
```

## Vulkan 与 OpenGL 切换

默认后端是 Vulkan。

在游戏的视频设置中可以切换渲染后端。由于窗口和图形上下文必须在启动阶段创建，修改后端后需要重启客户端。

也可以在仓库根目录的 `options.txt` 中手动设置：

```text
rustRenderBackend:vulkan
```

或：

```text
rustRenderBackend:opengl
```

未知值会安全回退到 Vulkan。

## 材质包

将 Minecraft Java Edition 1.12.2 材质包 ZIP 或文件夹放入：

```text
resourcepacks/
```

然后通过游戏中的资源包界面启用。资源包使用 Java 版标准 `assets/<namespace>/...` 结构，后选择的资源包覆盖运行时基础资源。

建议优先使用面向 1.12.2 或兼容 `pack_format: 3` 的资源包。高版本资源包可能包含本项目和原版 1.12.2 均不识别的模型、状态或纹理结构。

## OptiFine 光影

将光影包 ZIP 或包含 `shaders/` 的文件夹放入：

```text
shaderpacks/
```

使用步骤：

1. 将渲染后端切换为 OpenGL；
2. 重启客户端；
3. 打开“视频设置 → 光影”；
4. 选择光影包；
5. 根据需要调整 Shader Options。

当前光影路径面向 OptiFine 1.12.2 风格的：

- `gbuffers_*`；
- `composite*`；
- `final`；
- `shadow`；
- `world-1`、`world0`、`world1`；
- `shaders.properties`；
- include；
- Shader Options。

兼容性限制：

- Vulkan 后端不运行传统 OptiFine GLSL 光影；
- 不是所有第三方光影包都能保证兼容；
- 光影包依赖的非标准扩展、驱动行为或高版本 OptiFine 特性可能失败；
- 光影包自身的重复选项、无效配置和编译错误会保留为真实诊断，不会被静默隐藏。

## 调试与日志

客户端默认输出 INFO 日志。

提高日志级别：

```bat
set RUST_LOG=debug
cargo run --release
```

只查看特定模块：

```bat
set RUST_LOG=minecraft_1_12_2_rust_vulkan::vulkan=debug
cargo run --release
```

常见性能日志包括：

- 世界准备阶段；
- 可见区块和驻留区块；
- Vulkan/OpenGL 提交数量；
- TESR 与静态实体缓存复用；
- 透明重排；
- GUI 纹理上传。

高 FPS 并不自动证明行为正确。性能修改还必须检查透明顺序、实体层次、GUI状态、资源重载、世界切换和 OptiFine 程序边界。

## 当前边界

公开基线 v0.127.0 已经超出早期“仅远程多人客户端”阶段，但仍是持续迁移中的语义移植工程：

- 项目不是 Mojang 官方客户端，也不是 Minecraft 1.12.2 的完成版替代品；
- Flat IntegratedServer 已能真实创建/进入；方块和玩家状态的服务端权威保存链已开始闭合，但复杂 TileEntity、多方块、红石依赖交互仍需继续补齐；
- Default / Default 1.1 / Large Biomes / Amplified / Customized 已有真实 MCP 派生基础地形、biome surface、洞穴和峡谷；村庄、矿井、要塞、神殿、海底神殿、林地府邸以及湖泊、地牢、矿物、树木、花草等 population/decorator 仍在迁移；
- Nether、End 与 Debug generator 尚未达到完整 1.12.2 行为；
- 部分少见实体、TileEntity、交互或视觉边缘情况仍可能与原版存在差异；
- 任意第三方 OptiFine 光影包的普遍兼容性未作保证；
- 内置 Microsoft 登录依赖 Microsoft/Xbox/Minecraft 在线认证服务，服务端策略、二次验证或账号状态可能导致登录失败；
- `config/account.json` 保存明文令牌，只能保留在可信本地环境中；
- 仓库不捆绑原版资产，因此首次运行前必须从合法本地来源导入资源。

发现差异时，应提供：

1. Minecraft 1.12.2 原版表现；
2. 本项目表现；
3. 可复现步骤；
4. Vulkan 或 OpenGL 后端；
5. 使用的世界类型、资源包/光影包；
6. 完整日志；
7. 对应 MCP 类或方法（如可确定）。

## 开发与贡献约束

贡献应遵循以下顺序：

1. 先定位原版 1.12.2 MCP 类和调用链；
2. 检查现有 Rust 实现，避免重复重写；
3. 使用 Rust 等价结构实现原版语义；
4. 不引入假数据、占位几何或静态截图；
5. 保持网络、GUI、渲染层和状态顺序；
6. 同时检查 Vulkan 与 OpenGL；
7. 运行格式、Release 检查和测试；
8. 在真实客户端场景中进行对照验证。

RustCraft-Public 仅作为经过真实运行验证的渲染工程参考，用于比较区块调度、显存驻留、上传和提交策略。本仓库不包含 RustCraft 的源码文件、着色器、资源或品牌。Minecraft 1.12.2 MCP 始终是可见行为的权威基准。

## 法律声明

- 本项目与 Mojang Studios、Microsoft、MCP、OptiFine 均无官方隶属或认可关系。
- “Minecraft”及相关资产归其权利人所有。
- 本仓库不授予任何 Minecraft、MCP、OptiFine、RustCraft、Exhibition-Reborn、光影包或材质包的再分发权。
- 账号管理器的交互与行为参考 Exhibition-Reborn；仓库不包含其原始 Java 二进制、专有资源或品牌资产。
- 使用者必须自行拥有合法的 Minecraft 资源来源并遵守相关许可和服务条款。
- 仓库代码的使用权限以根目录 `LICENSE` 为准。

<img width="1920" height="1020" alt="QQ20260810-113451" src="https://github.com/user-attachments/assets/8fae411f-57e8-4885-ac72-a2be00c98538" />
<img width="1920" height="1020" alt="QQ20260810-113523" src="https://github.com/user-attachments/assets/c7bb67a2-85ec-452d-bce5-7aa01278c748" />
<img width="1920" height="1020" alt="QQ20260810-113639" src="https://github.com/user-attachments/assets/e1209daa-c777-40be-b244-2df2ed6ee1ff" />
<img width="1920" height="1020" alt="QQ20260810-113851" src="https://github.com/user-attachments/assets/07e50b4b-2a7b-4798-8877-77558c29da0b" />
随便传几张照片展示下效果罢了
