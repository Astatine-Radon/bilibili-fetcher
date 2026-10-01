const form = document.getElementById('dl-form');
const input = document.getElementById('url');
const submitBtn = document.getElementById('submit-btn');
const tasksEl = document.getElementById('tasks');

form.addEventListener('submit', async (e) => {
  e.preventDefault();
  const url = input.value.trim();
  if (!url) return;

  submitBtn.disabled = true;
  try {
    const res = await fetch('/api/tasks', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url }),
    });

    if (!res.ok) {
      alert('创建任务失败：' + (await res.text()));
      return;
    }

    const { task_id } = await res.json();
    input.value = '';
    createTaskCard(task_id, url);
    subscribeToTask(task_id);
  } catch (err) {
    alert('请求失败：' + err.message);
  } finally {
    submitBtn.disabled = false;
    input.focus();
  }
});

function createTaskCard(taskId, url) {
  const card = document.createElement('div');
  card.className = 'task';
  card.id = `task-${taskId}`;
  card.innerHTML = `
    <div class="task-header">
      <div class="task-title">${escapeHtml(url)}</div>
      <div class="task-status" data-status>排队中</div>
    </div>
    <div class="task-message" data-message>等待开始…</div>
    <div class="progress-wrap">
      <div class="progress-label">
        <span data-stage>准备中</span>
        <span data-percent></span>
      </div>
      <div class="progress-bar">
        <div class="progress-fill indeterminate" data-fill></div>
      </div>
    </div>
  `;
  tasksEl.prepend(card);
}

function subscribeToTask(taskId) {
  const card = document.getElementById(`task-${taskId}`);
  if (!card) return;

  const ui = {
    card,
    statusEl:  card.querySelector('[data-status]'),
    messageEl: card.querySelector('[data-message]'),
    stageEl:   card.querySelector('[data-stage]'),
    percentEl: card.querySelector('[data-percent]'),
    fillEl:    card.querySelector('[data-fill]'),
  };

  const es = new EventSource(`/api/tasks/${taskId}/events`);
  ui.es = es;

  es.onmessage = (e) => handleEvent(JSON.parse(e.data), ui);
  es.onerror = () => { /* 服务端结束后浏览器会重连，这里忽略 */ };
}

function handleEvent(ev, ui) {
  switch (ev.kind) {
    case 'queued':
      ui.statusEl.textContent = '排队中';
      break;

    case 'status':
      ui.messageEl.textContent = ev.message;
      break;

    case 'progress': {
      const stageName = ev.stage === 'video' ? '视频流' : '音频流';
      ui.stageEl.textContent = `正在下载 ${stageName}`;
      if (ev.total && ev.total > 0) {
        const pct = (ev.downloaded / ev.total) * 100;
        ui.fillEl.classList.remove('indeterminate');
        ui.fillEl.style.width = pct.toFixed(1) + '%';
        ui.percentEl.textContent =
          `${pct.toFixed(1)}% · ${fmtSize(ev.downloaded)} / ${fmtSize(ev.total)}`;
      } else {
        ui.fillEl.classList.add('indeterminate');
        ui.percentEl.textContent = fmtSize(ev.downloaded);
      }
      break;
    }

    case 'done': {
      ui.statusEl.textContent = '完成';
      ui.statusEl.classList.add('done');
      ui.messageEl.textContent = `文件大小：${fmtSize(ev.size)}`;
      ui.stageEl.textContent = '';
      ui.percentEl.textContent = '100%';
      ui.fillEl.classList.remove('indeterminate');
      ui.fillEl.style.width = '100%';

      const a = document.createElement('a');
      a.className = 'download-btn';
      a.href = `/api/tasks/${ui.card.id.replace('task-', '')}/file`;
      a.textContent = '⬇ 下载 MP4';
      a.setAttribute('download', ev.filename);
      ui.card.appendChild(a);

      setTimeout(() => ui.es.close(), 200);
      break;
    }

    case 'failed': {
      ui.statusEl.textContent = '失败';
      ui.statusEl.classList.add('error');
      ui.messageEl.textContent = ev.message;
      ui.stageEl.textContent = '';
      ui.percentEl.textContent = '';
      ui.fillEl.classList.remove('indeterminate');
      ui.fillEl.style.background = 'var(--error)';

      setTimeout(() => ui.es.close(), 200);
      break;
    }
  }
}

function fmtSize(bytes) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0;
  while (bytes >= 1024 && i < units.length - 1) { bytes /= 1024; i++; }
  return bytes.toFixed(i === 0 ? 0 : 1) + ' ' + units[i];
}

function escapeHtml(s) {
  return s.replace(/[&<>"']/g, (c) => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;',
    '"': '&quot;', "'": '&#39;'
  })[c]);
}