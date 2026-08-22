// 轻量级 i18n — 支持中文 / 英文，无需额外依赖
//
// 用法：
//   const { t, lang, setLang } = useI18n();
//   t("key")                       → 取当前语言的文案
//   t("key", { n: 3 })             → 替换 {n} 占位符
// 语言选择持久化到 localStorage（streamcap_lang）。

import { createContext, useContext, useState, useCallback, type ReactNode } from "react";

export type Lang = "zh" | "en";

type Dict = Record<string, string>;

const zh: Dict = {
  // 导航
  "nav.recordingList": "录制列表",
  "nav.addTask": "添加任务",
  "nav.files": "文件管理",
  "nav.history": "录制历史",
  "nav.settings": "设置",
  "nav.loading": "加载中...",

  // FFmpeg 状态
  "ffmpeg.available": "✅ FFmpeg 可用",
  "ffmpeg.missing": "❌ FFmpeg 未安装",
  "ffmpeg.checking": "检查 FFmpeg...",
  "ffmpeg.install": "一键安装 FFmpeg",
  "ffmpeg.installSuccess": "FFmpeg 安装成功",
  "ffmpeg.installFail": "FFmpeg 安装失败: {error}",

  // 录制状态
  "status.monitoring": "监控中",
  "status.checking": "检查中",
  "status.live": "直播中",
  "status.recording": "录制中",
  "status.retrying": "重试中",
  "status.offline": "离线",
  "status.error": "错误",
  "status.scheduled": "已排期",
  "status.queued": "排队中",

  // 录制卡片
  "card.unknownPlatform": "未识别",
  "card.waitDetect": "等待检测...",
  "card.startAt": "开始",
  "card.timeWindow": "录制时间窗口",
  "card.scheduledStart": "排期开始",
  "card.monitoring": "🔔 监控中",
  "card.paused": "⏸️ 已暂停",
  "card.pauseMonitor": "暂停监控",
  "card.startMonitor": "开始监控",
  "card.startRecording": "开始录制",
  "card.stopRecording": "停止录制",
  "card.delete": "删除",
  "card.edit": "编辑",
  "card.deleteConfirmTitle": "确定删除此录制任务？",
  "card.cancel": "取消",
  "card.retryMsg": "录制失败，第 {n} 次重试中...",
  "card.started": "录制已启动",
  "card.stopped": "录制已停止",
  "card.deleted": "已删除",
  "card.opFail": "{action}失败: {error}",

  // 操作名（用于 opFail 插值）
  "op.operation": "操作",
  "op.startRecord": "启动录制",
  "op.stopRecord": "停止录制",
  "op.delete": "删除",

  // 列表
  "list.searchPlaceholder": "搜索主播 / 标题 / 链接 / 平台",
  "list.emptyTitle": "暂无录制任务",
  "list.emptyAction": "点击添加任务开始监控直播",
  "list.noMatch": "没有匹配的录制任务",
  "filter.all": "全部",
  "filter.recording": "录制中",
  "filter.live": "直播中",
  "filter.monitoring": "监控中",
  "filter.retrying": "重试中",
  "filter.error": "错误",
  "filter.offline": "已停止",

  // 添加任务
  "add.urlLabel": "直播间 URL",
  "add.urlRequired": "请输入直播间 URL",
  "add.urlPlaceholder": "例如: https://live.bilibili.com/12345",
  "add.customStream": "🔗 自定义流 / M3U8 URL",
  "add.quality": "视频质量",
  "add.format": "输出格式",
  "add.timeWindowTitle": "录制时间窗口（可选）",
  "add.timeWindowHint": "留空 = 全天录制",
  "add.start": "开始",
  "add.end": "结束",
  "add.addTimeWindow": "添加时间窗口",
  "add.monitorNow": "添加后立即开始监控",
  "add.confirm": "确认添加",
  "add.success": "任务已添加",
  "add.fail": "添加失败: {error}",
  "add.scheduleRecordTitle": "定时录制",
  "add.scheduleRecordHint": "设置开始时间，到点后才开始监控 / 录制；可设为每天或每周重复",
  "add.scheduledStart": "开始时间",
  "add.scheduledStartPlaceholder": "选择开始时间（可选）",
  "add.recurrence": "重复方式",
  "add.recurrenceOnce": "仅一次",
  "add.recurrenceDaily": "每天",
  "add.recurrenceWeekly": "每周",

  // 编辑任务
  "edit.title": "编辑录制任务",
  "edit.anchorName": "主播名",
  "edit.anchorNamePlaceholder": "可选，留空则使用直播间默认名",
  "edit.titleLabel": "直播标题",
  "edit.titlePlaceholder": "可选，留空则使用直播间默认标题",
  "edit.urlLockedHint": "录制进行中，直播间地址已锁定（与正在进行的录制绑定）",
  "edit.save": "保存修改",
  "edit.success": "修改已保存",
  "edit.fail": "保存失败: {error}",

  // 平台名
  "platform.douyin": "抖音直播",
  "platform.bilibili": "哔哩哔哩直播",
  "platform.twitch": "Twitch",
  "platform.youtube": "YouTube",
  "platform.huya": "虎牙直播",
  "platform.kuaishou": "快手直播",
  "platform.douyu": "斗鱼直播",
  "platform.tiktok": "TikTok",
  "platform.rednote": "小红书直播",

  // 视频质量
  "quality.OD": "原画",
  "quality.UHD": "超清",
  "quality.HD": "高清",
  "quality.SD": "标清",
  "quality.LD": "低清",

  // 输出格式
  "format.ts": "TS (推荐)",
  "format.mp4": "MP4",
  "format.mkv": "MKV",
  "format.flv": "FLV",
  "format.mov": "MOV",

  // 设置
  "settings.outputDir": "输出目录",
  "settings.outputDirTooltip": "录制文件保存的根目录",
  "settings.outputDirPlaceholder": "录制文件保存路径",
  "settings.detectInterval": "检测间隔（秒）",
  "settings.detectIntervalTooltip": "建议 120-300 秒，过于频繁可能导致 IP 被封",
  "settings.diskThreshold": "磁盘空间阈值（GB）",
  "settings.diskThresholdTooltip": "低于此值自动停止录制，0 表示不限制",
  "settings.dirStructure": "目录结构",
  "settings.byPlatform": "按平台创建子文件夹",
  "settings.byAnchor": "按主播创建子文件夹",
  "settings.byDate": "按日期创建子文件夹",
  "settings.byTitle": "按标题创建子文件夹",
  "settings.segment": "录制分段",
  "settings.segmentDuration": "分段时长（秒）",
  "settings.segmentDurationTooltip": "录制达到指定时长后自动切换到新文件，0 表示不分段。默认 1800 秒（30 分钟）",
  "settings.retry": "失败重试",
  "settings.maxRetries": "最大重试次数",
  "settings.maxRetriesTooltip": "录制因网络/FFmpeg 等瞬时错误失败时自动重试，0 表示不重试。主播下播、用户停止、磁盘满不触发重试",
  "settings.retryDelay": "重试退避（秒）",
  "settings.retryDelayTooltip": "第 n 次重试前等待 retry_delay_seconds × n 秒，避免频繁重试",
  "settings.conversion": "格式转换",
  "settings.enableConversion": "启用录制后格式转换",
  "settings.enableConversionTooltip": "录制完成后自动转换为目标格式（流复制 remux，无重编码，速度快且无损）。默认关闭",
  "settings.targetFormat": "转换目标格式",
  "settings.targetFormatTooltip": "转换后的视频容器格式",
  "settings.deleteOriginal": "转换后删除原文件",
  "settings.deleteOriginalTooltip": "转换成功后自动删除原始录制文件。转换失败时始终保留原文件",
  "settings.proxy": "代理设置",
  "settings.enableProxy": "启用代理",
  "settings.proxyUrl": "代理地址",
  "settings.proxyUrlPlaceholder": "http://127.0.0.1:7890",
  "settings.cookie": "Cookie 配置",
  "settings.cookieHint": "按平台配置，留空则不使用",
  "settings.cookiePlaceholder": "粘贴 {label} 的 Cookie（如 SESSDATA=xxx; ...）",
  "settings.defaultQuality": "默认视频质量",
  "settings.defaults": "默认录制参数",
  "settings.defaultFormat": "默认格式",
  "settings.defaultFormatTooltip": "新建录制任务时使用的默认输出格式",
  "settings.save": "保存设置",
  "settings.saved": "设置已保存",
  "settings.saveFail": "保存失败: {error}",
  "settings.groupStorage": "存储与目录",
  "settings.groupNetwork": "网络与通知",
  "settings.concurrency": "并发录制",
  "settings.maxConcurrent": "最大同时录制路数",
  "settings.maxConcurrentTooltip": "0 = 不限制；超过上限的开播任务会排队，待空闲路数释放后再开录",
  "settings.cleanup": "磁盘自动清理",
  "settings.autoCleanup": "自动清理最旧的已完成录制",
  "settings.autoCleanupTooltip": "当可用空间低于「磁盘空间阈值」时，自动删除最旧的已完成录制以腾出空间（需阈值 > 0）",
  "settings.cleanupNow": "立即清理",
  "settings.cleanupDone": "已清理 {n} 个最旧录制",
  "settings.cleanupNothing": "暂无可清理的录制（空间充足或没有已完成录制）",
  "settings.cleanupFail": "清理失败: {error}",

  // 语言
  "lang.label": "语言",
  "lang.zh": "中文",
  "lang.en": "English",

  // 文件管理
  "files.up": "上级",
  "files.folderEmpty": "文件夹为空",
  "files.openFail": "打开失败: {error}",
  "files.loadFail": "加载文件列表失败: {error}",
  "files.enter": "进入",
  "files.open": "打开",
  "files.download": "下载",
  "files.name": "名称",
  "files.size": "大小",
  "files.modified": "修改时间",
  "files.action": "操作",

  // 录制历史
  "history.title": "录制历史",
  "history.empty": "暂无录制历史",
  "history.searchPlaceholder": "搜索主播 / 标题 / 平台",
  "history.anchor": "主播",
  "history.status": "状态",
  "history.started": "开始时间",
  "history.duration": "时长",
  "history.size": "文件大小",
  "history.play": "播放",
  "history.postprocess": "后处理",
  "history.delete": "删除记录",
  "history.deleteConfirm": "确定删除此历史记录？",
  "history.deleteFile": "同时删除文件",
  "history.deleted": "记录已删除",
  "history.loadFail": "加载历史失败: {error}",
  "history.status.completed": "已完成",
  "history.status.failed": "失败",
  "history.status.cancelled": "已取消",
  "history.errMsg": "错误: {error}",

  // 播放器
  "player.title": "播放",
  "player.openExternal": "在系统播放器中打开",
  "player.noFile": "无可用视频文件",
  "player.supportedHint": "桌面模式下请使用「在系统播放器中打开」",

  // 后处理
  "postprocess.title": "后处理",
  "postprocess.kind": "处理类型",
  "postprocess.convert": "格式转换",
  "postprocess.extractAudio": "提取音频",
  "postprocess.trim": "裁剪片段",
  "postprocess.targetFormat": "目标格式",
  "postprocess.startSeconds": "起始（秒）",
  "postprocess.endSeconds": "结束（秒，留空到结尾）",
  "postprocess.start": "开始处理",
  "postprocess.running": "处理中...",
  "postprocess.done": "处理完成",
  "postprocess.failed": "处理失败",
  "postprocess.progress": "进度",
  "postprocess.started": "已提交后处理任务",
  "postprocess.fail": "后处理失败: {error}",
  "postprocess.pickFile": "选择一个录制文件",

  // Webhook（设置）
  "settings.webhook": "事件通知 (Webhook)",
  "settings.webhookUrl": "Webhook URL",
  "settings.webhookUrlPlaceholder": "https://example.com/webhook",
  "settings.webhookTooltip": "录制开始 / 完成 / 失败时向该地址 POST 事件，留空则关闭通知",

  // 录制状态通知（toast）
  "notify.recordingStarted": "🔴 {name} 开始录制",
  "notify.recordingDone": "✅ {name} 录制完成",
  "notify.recordingFailed": "❌ {name} 录制失败",

  // 桌面体验
  "settings.groupDesktop": "桌面体验",
  "settings.groupFfmpeg": "FFmpeg",
  "settings.ffmpegPath": "FFmpeg 路径",
  "settings.ffmpegPathTooltip": "自定义 FFmpeg 可执行文件绝对路径；留空则使用 PATH 中的 ffmpeg。一键安装后会自动填入下载的静态二进制路径。",
  "settings.ffmpegPathPlaceholder": "留空使用系统 ffmpeg",
  "settings.ffmpegPathHint": "若未安装 FFmpeg，可前往侧边栏点击「一键安装 FFmpeg」自动下载静态二进制（无需 sudo），或在此手动指定已安装的路径。",
  "settings.minimizeToTray": "关闭窗口时最小化到托盘",
  "settings.minimizeToTrayTooltip": "关闭主窗口时不退出，保留在系统托盘继续录制与轮询；关闭则直接退出应用",
  "settings.autoLaunch": "开机自启",
  "settings.autoLaunchTooltip": "系统登录时自动启动（仅桌面端生效，server 模式忽略）",

  // 关闭遮罩
  "shutdown.done": "安全关闭完成",
  "shutdown.closing": "正在安全关闭应用",
  "shutdown.wait": "请稍候...",
  "shutdown.active": "检测到 {n} 个录制任务正在运行，正在发送停止信号并等待 FFmpeg 优雅退出...",
};

const en: Dict = {
  "nav.recordingList": "Recordings",
  "nav.addTask": "Add Task",
  "nav.files": "Files",
  "nav.history": "History",
  "nav.settings": "Settings",
  "nav.loading": "Loading...",

  "ffmpeg.available": "✅ FFmpeg Ready",
  "ffmpeg.missing": "❌ FFmpeg Not Installed",
  "ffmpeg.checking": "Checking FFmpeg...",
  "ffmpeg.install": "Install FFmpeg",
  "ffmpeg.installSuccess": "FFmpeg installed successfully",
  "ffmpeg.installFail": "FFmpeg install failed: {error}",

  "status.monitoring": "Monitoring",
  "status.checking": "Checking",
  "status.live": "Live",
  "status.recording": "Recording",
  "status.retrying": "Retrying",
  "status.offline": "Offline",
  "status.error": "Error",
  "status.scheduled": "Scheduled",
  "status.queued": "Queued",

  "card.unknownPlatform": "Unknown",
  "card.waitDetect": "Waiting for detection...",
  "card.startAt": "Start",
  "card.timeWindow": "Recording window",
  "card.scheduledStart": "Scheduled start",
  "card.monitoring": "🔔 Monitoring",
  "card.paused": "⏸️ Paused",
  "card.pauseMonitor": "Pause Monitor",
  "card.startMonitor": "Start Monitor",
  "card.startRecording": "Start Recording",
  "card.stopRecording": "Stop Recording",
  "card.delete": "Delete",
  "card.edit": "Edit",
  "card.deleteConfirmTitle": "Delete this recording task?",
  "card.cancel": "Cancel",
  "card.retryMsg": "Recording failed, retrying ({n})...",
  "card.started": "Recording started",
  "card.stopped": "Recording stopped",
  "card.deleted": "Deleted",
  "card.opFail": "{action} failed: {error}",

  "op.operation": "Operation",
  "op.startRecord": "Start recording",
  "op.stopRecord": "Stop recording",
  "op.delete": "Delete",

  "list.searchPlaceholder": "Search anchor / title / URL / platform",
  "list.emptyTitle": "No recordings yet",
  "list.emptyAction": "Click to add a task and start monitoring",
  "list.noMatch": "No matching recordings",
  "filter.all": "All",
  "filter.recording": "Recording",
  "filter.live": "Live",
  "filter.monitoring": "Monitoring",
  "filter.retrying": "Retrying",
  "filter.error": "Error",
  "filter.offline": "Stopped",

  "add.urlLabel": "Stream URL",
  "add.urlRequired": "Please enter the stream URL",
  "add.urlPlaceholder": "e.g. https://live.bilibili.com/12345",
  "add.customStream": "🔗 Custom stream / M3U8 URL",
  "add.quality": "Quality",
  "add.format": "Output Format",
  "add.timeWindowTitle": "Recording Window (optional)",
  "add.timeWindowHint": "Empty = record all day",
  "add.start": "Start",
  "add.end": "End",
  "add.addTimeWindow": "Add time window",
  "add.monitorNow": "Start monitoring immediately",
  "add.confirm": "Confirm",
  "add.success": "Task added",
  "add.fail": "Add failed: {error}",
  "add.scheduleRecordTitle": "Scheduled Recording",
  "add.scheduleRecordHint": "Set a start time; recording begins after it. Can repeat daily or weekly",
  "add.scheduledStart": "Start time",
  "add.scheduledStartPlaceholder": "Select start time (optional)",
  "add.recurrence": "Repeat",
  "add.recurrenceOnce": "Once",
  "add.recurrenceDaily": "Daily",
  "add.recurrenceWeekly": "Weekly",

  "edit.title": "Edit Recording Task",
  "edit.anchorName": "Anchor Name",
  "edit.anchorNamePlaceholder": "Optional, leave blank to use stream default",
  "edit.titleLabel": "Stream Title",
  "edit.titlePlaceholder": "Optional, leave blank to use stream default",
  "edit.urlLockedHint": "Recording in progress — stream URL is locked (bound to the active recording)",
  "edit.save": "Save Changes",
  "edit.success": "Changes saved",
  "edit.fail": "Save failed: {error}",

  "platform.douyin": "Douyin",
  "platform.bilibili": "Bilibili",
  "platform.twitch": "Twitch",
  "platform.youtube": "YouTube",
  "platform.huya": "Huya",
  "platform.kuaishou": "Kuaishou",
  "platform.douyu": "Douyu",
  "platform.tiktok": "TikTok",
  "platform.rednote": "Xiaohongshu",

  "quality.OD": "Original",
  "quality.UHD": "Ultra HD",
  "quality.HD": "HD",
  "quality.SD": "SD",
  "quality.LD": "LD",

  "format.ts": "TS (recommended)",
  "format.mp4": "MP4",
  "format.mkv": "MKV",
  "format.flv": "FLV",
  "format.mov": "MOV",

  "settings.outputDir": "Output Directory",
  "settings.outputDirTooltip": "Root directory for recorded files",
  "settings.outputDirPlaceholder": "Path to save recordings",
  "settings.detectInterval": "Check Interval (s)",
  "settings.detectIntervalTooltip": "Recommended 120-300s. Too frequent may get your IP banned",
  "settings.diskThreshold": "Disk Space Threshold (GB)",
  "settings.diskThresholdTooltip": "Stop recording below this value, 0 = unlimited",
  "settings.dirStructure": "Directory Structure",
  "settings.byPlatform": "Subfolder by platform",
  "settings.byAnchor": "Subfolder by anchor",
  "settings.byDate": "Subfolder by date",
  "settings.byTitle": "Subfolder by title",
  "settings.segment": "Segmentation",
  "settings.segmentDuration": "Segment Duration (s)",
  "settings.segmentDurationTooltip": "Switch to a new file after the given duration, 0 = no split. Default 1800s (30 min)",
  "settings.retry": "Retry on Failure",
  "settings.maxRetries": "Max Retries",
  "settings.maxRetriesTooltip": "Auto-retry on transient errors (network/FFmpeg). 0 = no retry. Not triggered for stream ended / user stop / disk full",
  "settings.retryDelay": "Retry Backoff (s)",
  "settings.retryDelayTooltip": "Wait retry_delay_seconds × n before the n-th retry",
  "settings.conversion": "Format Conversion",
  "settings.enableConversion": "Convert format after recording",
  "settings.enableConversionTooltip": "Remux after recording (stream copy, fast & lossless). Off by default",
  "settings.targetFormat": "Target Format",
  "settings.targetFormatTooltip": "Container format after conversion",
  "settings.deleteOriginal": "Delete original after conversion",
  "settings.deleteOriginalTooltip": "Delete the original after a successful conversion. Original is always kept on failure",
  "settings.proxy": "Proxy",
  "settings.enableProxy": "Enable proxy",
  "settings.proxyUrl": "Proxy URL",
  "settings.proxyUrlPlaceholder": "http://127.0.0.1:7890",
  "settings.cookie": "Cookie",
  "settings.cookieHint": "Configured per platform, empty = not used",
  "settings.cookiePlaceholder": "Paste {label} cookie (e.g. SESSDATA=xxx; ...)",
  "settings.defaultQuality": "Default Quality",
  "settings.defaults": "Default Recording",
  "settings.defaultFormat": "Default Format",
  "settings.defaultFormatTooltip": "Default output format for new recordings",
  "settings.save": "Save Settings",
  "settings.saved": "Settings saved",
  "settings.saveFail": "Save failed: {error}",
  "settings.groupStorage": "Storage & Directory",
  "settings.groupNetwork": "Network & Notifications",
  "settings.concurrency": "Concurrency",
  "settings.maxConcurrent": "Max concurrent recordings",
  "settings.maxConcurrentTooltip": "0 = unlimited; extra live tasks queue until a slot frees up",
  "settings.cleanup": "Auto Cleanup",
  "settings.autoCleanup": "Auto-delete oldest completed recordings",
  "settings.autoCleanupTooltip": "When free space drops below Disk Space Threshold, delete oldest completed recordings to free space (requires threshold > 0)",
  "settings.cleanupNow": "Clean Now",
  "settings.cleanupDone": "Cleaned up {n} oldest recording(s)",
  "settings.cleanupNothing": "Nothing to clean (enough space or no completed recordings)",
  "settings.cleanupFail": "Cleanup failed: {error}",

  "lang.label": "Language",
  "lang.zh": "中文",
  "lang.en": "English",

  "files.up": "Up",
  "files.folderEmpty": "Empty folder",
  "files.openFail": "Open failed: {error}",
  "files.loadFail": "Failed to load files: {error}",
  "files.enter": "Open",
  "files.open": "Open",
  "files.download": "Download",
  "files.name": "Name",
  "files.size": "Size",
  "files.modified": "Modified",
  "files.action": "Action",

  "history.title": "History",
  "history.empty": "No history yet",
  "history.searchPlaceholder": "Search anchor / title / platform",
  "history.anchor": "Anchor",
  "history.status": "Status",
  "history.started": "Started",
  "history.duration": "Duration",
  "history.size": "Size",
  "history.play": "Play",
  "history.postprocess": "Post-process",
  "history.delete": "Delete",
  "history.deleteConfirm": "Delete this history entry?",
  "history.deleteFile": "Also delete file",
  "history.deleted": "Entry deleted",
  "history.loadFail": "Failed to load history: {error}",
  "history.status.completed": "Completed",
  "history.status.failed": "Failed",
  "history.status.cancelled": "Cancelled",
  "history.errMsg": "Error: {error}",

  "player.title": "Playback",
  "player.openExternal": "Open in system player",
  "player.noFile": "No video file available",
  "player.supportedHint": "In desktop mode use \"Open in system player\"",

  "postprocess.title": "Post-process",
  "postprocess.kind": "Type",
  "postprocess.convert": "Convert format",
  "postprocess.extractAudio": "Extract audio",
  "postprocess.trim": "Trim clip",
  "postprocess.targetFormat": "Target format",
  "postprocess.startSeconds": "Start (sec)",
  "postprocess.endSeconds": "End (sec, empty = to end)",
  "postprocess.start": "Start",
  "postprocess.running": "Processing...",
  "postprocess.done": "Done",
  "postprocess.failed": "Failed",
  "postprocess.progress": "Progress",
  "postprocess.started": "Post-process job submitted",
  "postprocess.fail": "Post-process failed: {error}",
  "postprocess.pickFile": "Pick a recorded file",

  "settings.webhook": "Event Notification (Webhook)",
  "settings.webhookUrl": "Webhook URL",
  "settings.webhookUrlPlaceholder": "https://example.com/webhook",
  "settings.webhookTooltip": "POST events on recording start / complete / fail. Empty = disabled",

  "notify.recordingStarted": "🔴 {name} started recording",
  "notify.recordingDone": "✅ {name} recording finished",
  "notify.recordingFailed": "❌ {name} recording failed",

  // Desktop experience
  "settings.groupDesktop": "Desktop",
  "settings.groupFfmpeg": "FFmpeg",
  "settings.ffmpegPath": "FFmpeg Path",
  "settings.ffmpegPathTooltip": "Absolute path to the FFmpeg executable; leave empty to use PATH's ffmpeg. Auto-filled with the downloaded static binary after one-click install.",
  "settings.ffmpegPathPlaceholder": "Leave empty to use system ffmpeg",
  "settings.ffmpegPathHint": "If FFmpeg is missing, click \"Install FFmpeg\" in the sidebar to auto-download a static binary (no sudo required), or specify an installed path here manually.",
  "settings.minimizeToTray": "Minimize to tray on close",
  "settings.minimizeToTrayTooltip": "Keep running in the system tray (recording & polling continue) when the window is closed; turn off to quit on close",
  "settings.autoLaunch": "Launch at login",
  "settings.autoLaunchTooltip": "Start automatically on system login (desktop only; ignored in server mode)",

  "shutdown.done": "Shutdown complete",
  "shutdown.closing": "Safely shutting down...",
  "shutdown.wait": "Please wait...",
  "shutdown.active": "{n} recording task(s) running, sending stop signal and waiting for FFmpeg to exit...",
};

const resources: Record<Lang, Dict> = { zh, en };

interface I18nContextValue {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: string, vars?: Record<string, string | number>) => string;
}

const STORAGE_KEY = "streamcap_lang";

const I18nContext = createContext<I18nContextValue | null>(null);

export function LanguageProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    return saved === "en" ? "en" : "zh";
  });

  const setLang = useCallback((l: Lang) => {
    setLangState(l);
    localStorage.setItem(STORAGE_KEY, l);
  }, []);

  const t = useCallback(
    (key: string, vars?: Record<string, string | number>) => {
      let template = resources[lang][key] ?? resources.zh[key] ?? key;
      if (vars) {
        for (const [k, v] of Object.entries(vars)) {
          template = template.replace(new RegExp(`\\{${k}\\}`, "g"), String(v));
        }
      }
      return template;
    },
    [lang]
  );

  return (
    <I18nContext.Provider value={{ lang, setLang, t }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within LanguageProvider");
  return ctx;
}
