// StreamCap RS 前端逻辑

// 使用 Tauri API 与后端通信
const { invoke } = window.__TAURI__?.core || {};

// 状态
let currentPage = 'home';
let recordings = [];
let settings = {};

// ========================================
// 初始化
// ========================================
async function init() {
  // 检查 FFmpeg
  try {
    const version = await invoke('check_ffmpeg');
    document.getElementById('ffmpeg-status').textContent = '✅ FFmpeg 可用';
    document.getElementById('ffmpeg-status').style.color = '#10B981';
  } catch (e) {
    document.getElementById('ffmpeg-status').textContent = '❌ FFmpeg 未安装';
    document.getElementById('ffmpeg-status').style.color = '#EF4444';
  }

  // 加载数据
  await loadRecordings();
  await loadSettings();
}

// ========================================
// 导航
// ========================================
document.querySelectorAll('.nav-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    const page = btn.dataset.page;
    navigateTo(page);
  });
});

document.getElementById('btn-add')?.addEventListener('click', () => navigateTo('add'));

function navigateTo(page) {
  currentPage = page;

  // 更新导航按钮状态
  document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
  document.querySelector(`[data-page="${page}"]`)?.classList.add('active');

  // 切换页面
  document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
  document.getElementById(`page-${page}`)?.classList.add('active');

  // 页面特定初始化
  if (page === 'settings') loadSettingsForm();
}

// ========================================
// 录制列表
// ========================================
async function loadRecordings() {
  try {
    recordings = await invoke('list_recordings');
    renderRecordingList();
  } catch (e) {
    console.error('加载录制列表失败:', e);
  }
}

function renderRecordingList() {
  const container = document.getElementById('recording-list');
  if (!container) return;

  if (recordings.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <div class="empty-icon">📭</div>
        <p>暂无录制任务</p>
        <p class="empty-hint">点击右上角「添加任务」开始监控直播</p>
      </div>
    `;
    return;
  }

  container.innerHTML = recordings.map(r => {
    const status = getStatus(r);
    const statusClass = getStatusClass(status);
    const qualityLabel = r.quality?.label || r.quality || '原画';
    return `
      <div class="recording-card" id="card-${r.id}">
        <div class="card-header">
          <span class="card-platform">${r.platform || '未识别'}</span>
          <span class="card-status ${statusClass}">${statusLabel(status)}</span>
        </div>
        <div class="card-anchor">${r.anchor_name || r.url}</div>
        <div class="card-title">${r.title || '等待检测...'}</div>
        <div class="card-meta">
          <span>📹 ${qualityLabel}</span>
          <span>${r.monitor_enabled ? '🔔 监控中' : '⏸️ 已暂停'}</span>
        </div>
        <div class="card-actions">
          ${r.monitor_enabled
            ? `<button class="btn btn-sm btn-outline" data-action="monitor-off" data-id="${r.id}">⏸️ 暂停监控</button>`
            : `<button class="btn btn-sm btn-success" data-action="monitor-on" data-id="${r.id}">▶️ 开始监控</button>`
          }
          ${!r.is_recording && r.is_live
            ? `<button class="btn btn-sm btn-primary" data-action="record-start" data-id="${r.id}">🔴 开始录制</button>`
            : r.is_recording
              ? `<button class="btn btn-sm btn-danger" data-action="record-stop" data-id="${r.id}">⏹️ 停止录制</button>`
              : ''
          }
          <button class="btn btn-sm btn-outline" data-action="delete" data-id="${r.id}">🗑️ 删除</button>
        </div>
      </div>
    `;
  }).join('');
}

function getStatus(r) {
  if (r.is_recording) return 'recording';
  if (r.is_live) return 'live';
  if (r.monitor_enabled) return 'monitoring';
  if (r.error_message) return 'error';
  return 'offline';
}

function getStatusClass(status) {
  const map = {
    monitoring: 'status-monitoring',
    checking: 'status-checking',
    live: 'status-live',
    recording: 'status-recording',
    offline: 'status-offline',
    error: 'status-error',
  };
  return map[status] || 'status-offline';
}

function statusLabel(status) {
  const map = {
    monitoring: '监控中',
    checking: '检查中',
    live: '直播中',
    recording: '录制中',
    offline: '离线',
    error: '错误',
  };
  return map[status] || status;
}

// 事件委托：录制列表所有按钮统一处理
document.getElementById('recording-list')?.addEventListener('click', async (e) => {
  const btn = e.target.closest('button[data-action]');
  if (!btn) return;

  const action = btn.dataset.action;
  const id = btn.dataset.id;

  switch (action) {
    case 'monitor-on':
      await toggleMonitor(id, true);
      break;
    case 'monitor-off':
      await toggleMonitor(id, false);
      break;
    case 'record-start':
      await startRecord(id);
      break;
    case 'record-stop':
      await stopRecord(id);
      break;
    case 'delete':
      await deleteRecording(id);
      break;
  }
});

// 录制操作
async function toggleMonitor(id, enabled) {
  try {
    if (enabled) {
      await invoke('start_monitor', { id });
    } else {
      await invoke('stop_monitor', { id });
    }
    await loadRecordings();
  } catch (e) {
    alert('操作失败: ' + e);
  }
}

async function startRecord(id) {
  try {
    await invoke('start_recording', { id });
    await loadRecordings();
  } catch (e) {
    alert('启动录制失败: ' + e);
  }
}

async function stopRecord(id) {
  try {
    await invoke('stop_recording', { id });
    await loadRecordings();
  } catch (e) {
    alert('停止录制失败: ' + e);
  }
}

async function deleteRecording(id) {
  if (!confirm('确定删除此录制任务？')) return;
  try {
    await invoke('remove_recording', { id });
    await loadRecordings();
  } catch (e) {
    alert('删除失败: ' + e);
  }
}

// ========================================
// 添加任务
// ========================================
document.getElementById('input-url')?.addEventListener('input', (e) => {
  const url = e.target.value.trim();
  const hint = document.getElementById('platform-hint');
  if (!hint) return;

  // 简单平台检测
  const detections = [
    ['douyin.com', '🎵 抖音直播'],
    ['bilibili.com', '📺 哔哩哔哩直播'],
    ['twitch.tv', '🎮 Twitch'],
    ['youtube.com', '▶️ YouTube'],
    ['huya.com', '🐯 虎牙直播'],
    ['kuaishou.com', '📱 快手直播'],
    ['douyu.com', '🐟 斗鱼直播'],
    ['tiktok.com', '🎵 TikTok'],
    ['xiaohongshu.com', '📕 小红书直播'],
  ];

  const found = detections.find(([pattern]) => url.includes(pattern));
  hint.textContent = found ? found[1] : (url ? '🔗 自定义流 / M3U8 URL' : '');
});

document.getElementById('btn-submit')?.addEventListener('click', async () => {
  const url = document.getElementById('input-url')?.value.trim();
  if (!url) {
    alert('请输入直播间 URL');
    return;
  }

  const quality = document.getElementById('input-quality')?.value || 'OD';
  const monitor = document.getElementById('input-monitor')?.checked ?? true;

  try {
    await invoke('add_recording', { url, monitorEnabled: monitor, quality });
    await loadRecordings();
    navigateTo('home');

    // 清空表单
    document.getElementById('input-url').value = '';
  } catch (e) {
    alert('添加失败: ' + e);
  }
});

// ========================================
// 设置
// ========================================
async function loadSettings() {
  try {
    settings = await invoke('get_settings');
  } catch (e) {
    console.error('加载设置失败:', e);
  }
}

function loadSettingsForm() {
  document.getElementById('setting-output-dir').value = settings.output_dir || '';
  document.getElementById('setting-loop-interval').value = settings.loop_interval_seconds || 180;
  document.getElementById('setting-space-threshold').value = settings.recording_space_threshold_gb || 0;
  document.getElementById('setting-folder-platform').checked = settings.folder_by_platform || false;
  document.getElementById('setting-folder-anchor').checked = settings.folder_by_anchor || false;
  document.getElementById('setting-folder-date').checked = settings.folder_by_date ?? true;
  document.getElementById('setting-folder-title').checked = settings.folder_by_title || false;
  document.getElementById('setting-enable-proxy').checked = settings.enable_proxy || false;
  document.getElementById('setting-proxy-url').value = settings.proxy_url || '';
}

document.getElementById('btn-save-settings')?.addEventListener('click', async () => {
  const newSettings = {
    ...settings,
    output_dir: document.getElementById('setting-output-dir').value,
    loop_interval_seconds: parseInt(document.getElementById('setting-loop-interval').value) || 180,
    recording_space_threshold_gb: parseInt(document.getElementById('setting-space-threshold').value) || 0,
    folder_by_platform: document.getElementById('setting-folder-platform').checked,
    folder_by_anchor: document.getElementById('setting-folder-anchor').checked,
    folder_by_date: document.getElementById('setting-folder-date').checked,
    folder_by_title: document.getElementById('setting-folder-title').checked,
    enable_proxy: document.getElementById('setting-enable-proxy').checked,
    proxy_url: document.getElementById('setting-proxy-url').value,
  };

  try {
    await invoke('update_settings', { settings: newSettings });
    settings = newSettings;
    alert('设置已保存');
  } catch (e) {
    alert('保存失败: ' + e);
  }
});

// 代理复选框联动
document.getElementById('setting-enable-proxy')?.addEventListener('change', (e) => {
  const proxyInput = document.getElementById('setting-proxy-url');
  if (e.target.checked) {
    proxyInput.classList.add('active');
  } else {
    proxyInput.classList.remove('active');
  }
});

// ========================================
// 启动
// ========================================
document.addEventListener('DOMContentLoaded', init);
