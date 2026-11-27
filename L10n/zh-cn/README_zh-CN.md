[English](../../README.md) | **简体中文** | [Español](../es/README_es.md) | [Português (BR)](../pt-BR/README_pt-BR.md) | [日本語](../ja/README_ja.md) | [Русский](../ru/README_ru.md) | [Français](../fr/README_fr.md) | [Deutsch](../de/README_de.md) | [한국어](../ko/README_ko.md) | [Türkçe](../tr/README.tr.md) | [Bahasa Indonesia](../id/README.id.md)

# FamTree

作者与维护者：**[john3sobieski](https://github.com/john3sobieski)**。仓库地址：<https://github.com/john3sobieski/family-tree>

一款用于创建、编辑和浏览家谱的 Windows 桌面应用，另有可在手机上查看家谱的 Android 应用。所有内容都保存在单个 `.famtree` 文件中：人物、关系和全部照片。

> **状态：** 可以使用。项目视为已完成，今后只会有小的修复。请在 [Releases](https://github.com/john3sobieski/family-tree/releases) 页面下载 Windows 安装程序或 Android APK（见[安装](#安装)）。

## 功能

- **三种模式**
  - **查看**– 只读的思维导图，支持平滑缩放和平移。
  - **编辑**– 添加和修改人物、照片与关系。
  - **列表**– 可排序、可筛选的全体人物表格。
- **以你为中心的思维导图。** 将某个人标记为"我"，家谱就会从你最上层的祖先向下绘制，因此父母、兄弟姐妹、叔伯姑姨、堂表兄弟姐妹以及更远的亲属都会自动显示。父系和母系并排排布，姻亲家庭也会一并纳入。
- **照片卡片。** 每张卡片显示证件照风格的照片、名字、姓氏（女性的娘家姓写在括号里）、父称、表示性别的色条，以及最多两面国籍旗帜。
- **人物窗口。** 点击卡片会打开更大的窗口，包含完整照片、相册（每人 10 张以上照片没问题）、日期、年龄、出生地和逝世地、职业、备注，以及可点击跳转的父母、伴侣和子女链接。
- **搜索。** 输入名字后，镜头会飞到该人物所在位置并将其高亮。在列表模式下则作为筛选器使用。`Ctrl+F` 可聚焦搜索框。
- **关系。** 父母、伴侣、子女和兄弟姐妹可以一键添加，也可以关联到已有的人物。循环关系和重复关联会被拒绝。
- **不完整日期。** 只知道年份？没问题，日期的任何部分都可以留空。
- **国籍。** 每人最多可从内置的 197 个国家列表中选择两个（所有联合国会员国，加上梵蒂冈、巴勒斯坦、台湾和科索沃）。国旗由应用自行绘制。
- **为大型家谱而生。** 只绘制可见的卡片，缩略图延迟加载，只有结构变化时才重新计算布局。已在数千人规模的家谱上测试过。
- **自动保存。** 家谱一旦有了文件，停止输入片刻后每次修改都会自动保存。新建的家谱请先使用**另存为…**。如果有内容未能保存，关闭前也会提示你。
- **界面语言。** 支持 Русский、English、中文、Español、Français、Deutsch 和 Polski。通过**语言**菜单切换，所选语言会被记住。首次启动时应用会跟随 Windows 显示语言，若不在上述语言之列则使用英语。国家名称也会一并翻译。
- **导出。** **文件 → 导出…** 可将整棵树保存为 PNG 或矢量 PDF，四周留有边距。导出 PNG 时可选择背景（透明、白色或黑色）以及深色或浅色配色；尺寸会自动选择以保持清晰（最高 ×2）。PDF 可无损缩放。非常大的树更适合导出为 PDF。
- **双击打开。** 在**文件**菜单中可以将 `.famtree` 文件与应用关联。
- **深色和浅色主题。** 使用状态栏右端的太阳/月亮按钮切换，选择会被记住。
- **Android 应用（仅查看）。** 在手机或平板上打开同一个 `.famtree` 文件。地图与电脑上完全一致（相同的布局、卡片、国旗和连线），支持拖动和双指缩放、搜索并让镜头飞向该人物、带照片和亲属链接的全屏人物页面、**适应窗口**，以及浅色/深色主题。文件绝不会被修改：编辑、列表模式和导出仍在电脑上进行。界面跟随手机语言（同样的七种语言，其他语言显示英文）。见[安装 → Android](#android)。

## 安装

### Windows

应用可在 Windows 10 和 Windows 11 上运行，两者使用同一个安装程序和同一个 exe。

1. 打开 [Releases](https://github.com/john3sobieski/family-tree/releases) 页面，下载 **`FamTree-Setup-<版本>.exe`**。
2. 运行它。安装仅针对当前用户，不需要管理员权限。向导中可以选择用 FamTree 打开 `.famtree` 文件，以及添加桌面快捷方式。
3. 从开始菜单启动 **FamTree**。

注意事项：

- 由于安装程序和 `.exe` 未签名，Windows 可能会显示 SmartScreen 警告。请选择*更多信息 → 仍要运行*。
- 更新时，在旧版本上直接运行新的 Setup 即可。你的家谱和设置会保留。
- 卸载请使用*设置 → 应用*。你的 `.famtree` 文件不会被删除。
- 如果使用便携版 exe 并移动了它，请再次点击**文件**菜单中的 `.famtree` 关联按钮。

### Android

Android 应用只用于查看家谱，绝不会修改你的文件。需要 Android 10 或更高版本、64 位 ARM 设备（几乎所有当前的手机和平板都是）。

1. 在 [Releases](https://github.com/john3sobieski/family-tree/releases) 页面把 **`FamTree-<版本号>.apk`** 直接下载到设备上。
2. 打开该文件。Android 会询问一次是否允许从此来源（浏览器或文件管理器）安装应用，请确认。
3. 把 `.famtree` 文件放到设备上（云存储、即时通讯工具、USB 数据线均可）。然后在文件管理器中用 **打开方式 → FamTree** 打开，或启动 FamTree 并点按 **打开** 来选择文件。应用会记住最近打开的文件。应用运行时如果再打开另一个 `.famtree`，无需重启即可加载。

说明：

- 应用只以 APK 形式发布，不上架 Google Play。Play Protect 可能会提示来自未知开发者的应用，请选择仍然安装。
- 为了能再次打开，应用会保存最近打开文件的一份私有副本；原文件绝不会被改动。

### 自行构建

无需安装任何工具，也可以从 GitHub Actions 获取安装程序（需要 GitHub 账号）：**Fork** [仓库](https://github.com/john3sobieski/family-tree)，打开 **Actions → Build → Run workflow**，然后从完成的运行记录中下载 **`famtree-setup`** 产物（默认保留 90 天）。

Android APK 的获取方式相同：**Actions → Android → Run workflow**，然后下载 **`famtree-android-<版本号>`** 构建产物。

## 快速上手

1. 启动 `famtree.exe`。使用 **文件 → 演示**查看示例家谱，或使用 **文件 → 新建**开始你自己的家谱。
2. 切换到**编辑**模式，添加第一个人物（**＋ 新建人物**）。在空家谱中，此人会自动成为"我"。
3. 选中一个人，使用 **＋ 父母**、**＋ 伴侣**、**＋ 子女** 或 **＋ 兄弟姐妹**来扩展家谱。在右侧填写姓名、日期、照片和备注。
4. 选择**另存为…**并取一个名字。此后，修改会自动保存。
5. 切换到**查看**来浏览导图：拖动平移，滚轮缩放，**适应窗口**可查看全部内容，点击卡片查看详情。

**在 Android 上：** 打开 `.famtree` 文件（见[安装 → Android](#android)），拖动平移，双指缩放。点按 **适应窗口** 可看到全貌，点放大镜搜索人物，点按卡片打开人物页面（点按亲属可跳转到该人）。系统的返回键或返回手势会逐级关闭页面或搜索；在地图上则让应用退到后台。太阳/月亮按钮用于切换主题（默认跟随系统主题）。

## `.famtree` 文件

`.famtree` 文件就是一个普通的 ZIP 压缩包：

```
tree.json            people and relationships (plain, human-readable text)
media/<hash>.jpg     photos, resized to at most 1600 px
thumb/<hash>.jpg     small thumbnails for cards
```

- 照片存放在文件内部，所以你可以把一个 `.famtree` 发给亲戚而不会丢失任何内容。相同的照片只会存储一次。
- `tree.json` 可以用任何文本编辑器打开，并带有说明各个关系的注释。如果想手动编辑，请先解压压缩包，编辑文件后再重新打包。直接在 `.famtree` 内部编辑会破坏压缩包。
- 旧文件可以在新版本的应用中继续打开。

## 已知限制

- 除俄语外，界面翻译尚未经过母语者审校，欢迎通过 issue 提出修正。
- 无论使用哪种语言，日期始终显示为 `DD.MM.YYYY` 格式。
- 暂无撤销功能。
- 一次性导入大量照片时，窗口可能会短暂卡住。
- 不支持 HEIC 照片（支持 JPEG/JFIF、PNG、WebP 和 BMP）。
- 卡片上的缩略图是居中裁剪，暂时无法选择裁剪区域。
- 在非常大的家族中，布局仍可能把人物放到奇怪的位置，或使不同分支之间的连线交叉（见路线图）。
- 列表模式下不显示国籍。
- Android：仅查看（无编辑、列表模式和导出），支持 Android 10 或更高版本的 64 位 ARM 设备，仅提供 APK（不在 Google Play 上架）。
- Android：照片暂时不能用双指放大，也没有语言菜单：应用跟随系统语言。

## 路线图

项目已完成：Windows 应用和 Android 查看器均已就绪。今后只计划做小的修复，主要是针对偶尔出现的布局问题（人数很多的大家族中有人被放在奇怪的位置）。

## 从源码构建

如果你想在本地构建，需要在 Windows 上安装 Rust 工具链：

```
cargo test --release
cargo build --release
```

产物位于 `target/release/famtree.exe`。该项目的界面使用 [egui/eframe](https://github.com/emilk/egui)。

Android 应用（`android/`）只由 GitHub Actions 工作流 `.github/workflows/android.yml` 构建（cargo-apk、Android SDK/NDK、一小层 Java 代码和签名）。它直接编译 `src/` 中的桌面端代码（`model`、`container`、`layout`、`view`、`flags` 等），因此两个平台上的地图完全一致。如需用自己的密钥给 APK 签名，请添加仓库密钥 `ANDROID_KEYSTORE_BASE64`、`ANDROID_KEYSTORE_PASSWORD`、`ANDROID_KEY_ALIAS`，以及（若与前者不同）`ANDROID_KEY_PASSWORD`；未设置时，工作流会使用临时密钥。

## 反馈问题

发现了问题，尤其是导图中人物位置的问题？请[提交 issue](https://github.com/john3sobieski/family-tree/issues)，如果可以，请附上一个能复现问题的小型 `.famtree` 文件（如果家谱涉及隐私，请去除真实个人数据）。如果问题出在 Android 应用上，请同时说明 Android 版本和设备型号。

## 许可证

FamTree 采用 [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/) 许可。详见 [`LICENSE.md`](LICENSE.md)。

简而言之：

- 你可以出于**非商业**目的（个人使用、家族史研究、兴趣爱好、教育、科研等）使用、复制、修改和分享本应用及其源代码。
- **禁止出售或用于商业用途。**
- 每当你分享本应用或其修改版本（fork、重新构建、再分发）时，必须保留许可证条款以及 `LICENSE.md` 中的 `Required Notice` 行，其中注明了 john3sobieski 为作者。

以上摘要不构成法律意见，以许可证原文为准。
