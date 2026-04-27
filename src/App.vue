<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
}

interface FileState extends FileInfo {
  baseName: string;
  ext: string;
  editValue: string;
  status: 'pending' | 'success' | 'error';
  errMsg?: string;
}

function splitFileName(filename: string) {
  const lastDotIdx = filename.lastIndexOf('.');
  if (lastDotIdx > 0 && lastDotIdx < filename.length - 1) {
    return {
      baseName: filename.substring(0, lastDotIdx),
      ext: filename.substring(lastDotIdx)
    };
  }
  return { baseName: filename, ext: '' };
}

const currentDir = ref('');
const files = ref<FileState[]>([]);
const prefix = ref('');
const suffix = ref('');
const replaceTarget = ref('');
const replaceWith = ref('');
const isProcessing = ref(false);
const showExtension = ref(false);

const toastMsg = ref('');
let toastTimer: any = null;

const showToast = (msg: string) => {
  toastMsg.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toastMsg.value = '';
  }, 2000);
};

const selectFolder = async () => {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  
  if (selected) {
    currentDir.value = selected as string;
    await loadDirectory(currentDir.value);
  }
};

const loadDirectory = async (dirPath: string) => {
  try {
    const res: FileInfo[] = await invoke('list_files', { dirPath });
    files.value = res.map(f => {
      const { baseName, ext } = splitFileName(f.name);
      return {
        ...f,
        baseName,
        ext,
        editValue: showExtension.value ? f.name : baseName,
        status: 'pending'
      };
    });
    // 清空操作区
    prefix.value = '';
    suffix.value = '';
    replaceTarget.value = '';
    replaceWith.value = '';
  } catch (e) {
    console.error('Failed to load dir:', e);
    alert('加载目录失败: ' + e);
  }
};

// 监听后缀名显示开关
watch(showExtension, (newVal) => {
  files.value.forEach(f => {
    if (newVal) {
      f.editValue = f.editValue + f.ext;
    } else {
      const { baseName, ext } = splitFileName(f.editValue);
      if (ext) f.ext = ext; 
      f.editValue = baseName;
    }
  });
});

const applyPrefix = () => {
  if (!prefix.value) return;
  files.value.forEach(f => {
    f.editValue = prefix.value + f.editValue;
  });
};

const applySuffix = () => {
  if (!suffix.value) return;
  files.value.forEach(f => {
    if (showExtension.value) {
      const { baseName, ext } = splitFileName(f.editValue);
      f.editValue = baseName + suffix.value + ext;
    } else {
      f.editValue = f.editValue + suffix.value;
    }
  });
};

const applyReplace = () => {
  if (!replaceTarget.value) return;
  files.value.forEach(f => {
    f.editValue = f.editValue.split(replaceTarget.value).join(replaceWith.value);
  });
};

const handlePaste = (e: ClipboardEvent, startIdx: number) => {
  e.preventDefault();
  const text = e.clipboardData?.getData('text');
  if (!text) return;
  
  const lines = text.split(/\r?\n/).filter(l => l.trim() !== '');
  let currIdx = startIdx;
  
  for (const line of lines) {
    if (currIdx < files.value.length) {
      files.value[currIdx].editValue = line.trim();
      currIdx++;
    } else {
      break;
    }
  }
  showToast('单元格粘贴成功');
};

const copyOriginalNames = async () => {
  if (files.value.length === 0) return;
  const text = files.value.map(f => showExtension.value ? f.name : f.baseName).join('\n');
  try {
    await navigator.clipboard.writeText(text);
    showToast('已复制原文件名');
  } catch(e) {
    const textArea = document.createElement("textarea");
    textArea.value = text;
    document.body.appendChild(textArea);
    textArea.select();
    document.execCommand("copy");
    document.body.removeChild(textArea);
    showToast('已复制原文件名');
  }
};

const copyNewNames = async () => {
  if (files.value.length === 0) return;
  const text = files.value.map(f => showExtension.value ? f.editValue : f.editValue + f.ext).join('\n');
  try {
    await navigator.clipboard.writeText(text);
    showToast('已复制新文件名');
  } catch(e) {
    const textArea = document.createElement("textarea");
    textArea.value = text;
    document.body.appendChild(textArea);
    textArea.select();
    document.execCommand("copy");
    document.body.removeChild(textArea);
    showToast('已复制新文件名');
  }
};

const pasteFromClipboard = async () => {
  try {
    const text = await navigator.clipboard.readText();
    if (!text) return;
    
    const lines = text.split(/\r?\n/).filter(l => l.trim() !== '');
    let currIdx = 0;
    let pastedCount = 0;
    
    for (const line of lines) {
      if (currIdx < files.value.length) {
        files.value[currIdx].editValue = line.trim();
        currIdx++;
        pastedCount++;
      } else {
        break;
      }
    }
    showToast(`成功粘贴 ${pastedCount} 条名称`);
  } catch(e) {
    alert('读取剪贴板失败。由于安全限制，请在表格单元格内使用 Ctrl+V 进行粘贴。' + e);
  }
};

const resetChanges = () => {
  prefix.value = '';
  suffix.value = '';
  replaceTarget.value = '';
  replaceWith.value = '';
  files.value.forEach(f => {
    f.editValue = showExtension.value ? f.name : f.baseName;
    f.status = 'pending';
    f.errMsg = undefined;
  });
  showToast('修改已重置');
};

const executeRename = async () => {
  if (files.value.length === 0) return;
  isProcessing.value = true;
  
  const tasks = files.value.map(f => {
    const newName = showExtension.value ? f.editValue : f.editValue + f.ext;
    const dirSeparator = currentDir.value.includes('\\') ? '\\' : '/';
    const newPath = `${currentDir.value}${dirSeparator}${newName}`;
    return {
      old_path: f.path,
      new_path: newPath
    };
  });
  
  try {
    const results: any[] = await invoke('rename_files', { tasks });
    
    // 更新状态
    let successCount = 0;
    results.forEach((res, idx) => {
      if (res.success) {
        files.value[idx].status = 'success';
        const newName = showExtension.value ? files.value[idx].editValue : files.value[idx].editValue + files.value[idx].ext;
        files.value[idx].name = newName;
        files.value[idx].path = res.new_path;
        
        // 重新同步 baseName 和 ext
        const { baseName, ext } = splitFileName(newName);
        files.value[idx].baseName = baseName;
        files.value[idx].ext = ext;
        successCount++;
      } else {
        files.value[idx].status = 'error';
        files.value[idx].errMsg = res.error;
      }
    });
    showToast(`重命名完成: ${successCount} 成功 / ${results.length} 总计`);
  } catch(e) {
    console.error('Rename failed', e);
    alert('重命名执行异常: ' + e);
  } finally {
    isProcessing.value = false;
  }
};
</script>

<template>
  <div class="h-screen overflow-hidden bg-background text-foreground p-6 flex flex-col font-sans select-none relative">
    
    <!-- Global Toast -->
    <transition 
      enter-active-class="transition duration-300 ease-out" 
      enter-from-class="transform translate-y-4 opacity-0" 
      enter-to-class="transform translate-y-0 opacity-100" 
      leave-active-class="transition duration-200 ease-in" 
      leave-from-class="transform translate-y-0 opacity-100" 
      leave-to-class="transform translate-y-4 opacity-0"
    >
      <div v-if="toastMsg" class="fixed bottom-10 left-1/2 -translate-x-1/2 z-50 bg-neutral-800/90 backdrop-blur-md text-white px-6 py-2.5 rounded-full shadow-2xl text-sm border border-neutral-700/50 flex items-center gap-2">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-green-400"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
        {{ toastMsg }}
      </div>
    </transition>

    <!-- Header Area -->
    <header class="shrink-0 flex justify-between items-center pb-6 border-b border-primary-hover mb-6">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight text-white">Batch Rename Tool</h1>
        <p class="text-sm text-secondary mt-1">极简・极速・强大的批量重命名工具</p>
      </div>
      <div class="flex items-center gap-3">
        <button v-if="files.length > 0" @click="resetChanges" class="px-4 py-2.5 bg-primary-hover text-secondary hover:text-white font-medium rounded-md hover:bg-neutral-800 transition-colors shadow-sm text-sm focus:outline-none flex items-center gap-2 border border-primary-hover hover:border-neutral-700">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
          重置
        </button>
        <button 
          @click="selectFolder"
          class="px-5 py-2.5 bg-foreground text-background font-medium rounded-md hover:bg-neutral-200 transition-colors shadow-sm text-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-background focus:ring-foreground"
        >
          选择文件夹
        </button>
      </div>
    </header>

    <!-- Path display & Operations -->
    <div class="shrink-0 mb-6 space-y-4" v-if="currentDir">
      <div class="flex items-center justify-between text-sm text-secondary bg-primary/20 px-4 py-2 rounded-md border border-primary-hover">
        <div class="flex items-center space-x-2 truncate">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/></svg>
          <span class="truncate">{{ currentDir }}</span>
        </div>
        <div class="flex items-center space-x-2 shrink-0 ml-4">
          <span class="text-xs">显示扩展名</span>
          <button 
            @click="showExtension = !showExtension"
            class="relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none"
            :class="showExtension ? 'bg-foreground' : 'bg-primary-hover'"
          >
            <span 
              class="inline-block h-3 w-3 transform rounded-full bg-background transition-transform"
              :class="showExtension ? 'translate-x-5' : 'translate-x-1'"
            />
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="flex space-x-2">
          <input v-model="prefix" type="text" placeholder="加前缀" class="flex-1 bg-primary border border-primary-hover rounded px-3 py-1.5 text-sm focus:outline-none focus:border-neutral-500 transition-colors" />
          <button @click="applyPrefix" class="px-3 py-1.5 bg-primary-hover border border-primary-hover rounded hover:border-neutral-500 text-sm transition-colors text-white">应用</button>
        </div>
        <div class="flex space-x-2">
          <input v-model="suffix" type="text" placeholder="加后缀" class="flex-1 bg-primary border border-primary-hover rounded px-3 py-1.5 text-sm focus:outline-none focus:border-neutral-500 transition-colors" />
          <button @click="applySuffix" class="px-3 py-1.5 bg-primary-hover border border-primary-hover rounded hover:border-neutral-500 text-sm transition-colors text-white">应用</button>
        </div>
        <div class="flex space-x-2">
          <input v-model="replaceTarget" type="text" placeholder="查内容" class="w-1/3 bg-primary border border-primary-hover rounded px-2 py-1.5 text-sm focus:outline-none focus:border-neutral-500 transition-colors" />
          <input v-model="replaceWith" type="text" placeholder="替换为" class="w-1/3 bg-primary border border-primary-hover rounded px-2 py-1.5 text-sm focus:outline-none focus:border-neutral-500 transition-colors" />
          <button @click="applyReplace" class="px-3 py-1.5 bg-primary-hover border border-primary-hover rounded hover:border-neutral-500 text-sm transition-colors flex-1 text-white">替换</button>
        </div>
      </div>
    </div>

    <!-- Table Area -->
    <div class="flex-1 overflow-auto rounded-lg border border-primary-hover bg-primary/30" v-if="files.length > 0">
      <table class="w-full text-left text-sm whitespace-nowrap">
        <thead class="bg-primary text-secondary sticky top-0 z-10 shadow-sm border-b border-primary-hover">
          <tr>
            <th class="px-4 py-3 font-medium w-16">#</th>
            <th class="px-4 py-3 font-medium w-1/3">
              <div class="flex items-center justify-between">
                <span>原文件名</span>
                <button @click="copyOriginalNames" class="text-secondary hover:text-white transition-all text-xs flex items-center gap-1" title="复制全部">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                </button>
              </div>
            </th>
            <th class="px-4 py-3 font-medium w-1/3">
              <div class="flex items-center justify-between">
                <span>新文件名 {{ showExtension ? '(含后缀)' : '(不含后缀)' }}</span>
                <div class="flex items-center gap-2">
                  <button @click="pasteFromClipboard" class="text-secondary hover:text-white transition-all text-xs flex items-center gap-1" title="一键粘贴">
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path><rect x="8" y="2" width="8" height="4" rx="1" ry="1"></rect></svg>
                  </button>
                  <button @click="copyNewNames" class="text-secondary hover:text-white transition-all text-xs flex items-center gap-1" title="复制全部">
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                  </button>
                </div>
              </div>
            </th>
            <th class="px-4 py-3 font-medium">状态</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-primary-hover">
          <tr v-for="(file, idx) in files" :key="file.path" class="hover:bg-primary-hover/50 transition-colors group">
            <td class="px-4 py-3 text-secondary">{{ idx + 1 }}</td>
            <td class="px-4 py-3 truncate max-w-[200px]" :title="showExtension ? file.name : file.baseName">
              {{ showExtension ? file.name : file.baseName }}
            </td>
            <td class="px-4 py-2">
              <input 
                v-model="file.editValue" 
                @paste="(e) => handlePaste(e, idx)"
                type="text" 
                class="w-full bg-transparent border-b border-transparent group-hover:border-neutral-600 focus:border-neutral-400 focus:outline-none px-1 py-1 transition-colors text-white"
                :title="file.editValue"
              />
            </td>
            <td class="px-4 py-3">
              <span v-if="file.status === 'pending'" class="text-secondary text-xs px-2 py-1 rounded bg-primary">等待</span>
              <span v-else-if="file.status === 'success'" class="text-green-500 text-xs px-2 py-1 rounded bg-green-500/10">成功</span>
              <span v-else class="text-red-400 text-xs px-2 py-1 rounded bg-red-500/10" :title="file.errMsg">失败</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty State -->
    <div v-else class="flex-1 flex flex-col items-center justify-center border border-dashed border-primary-hover rounded-lg text-secondary p-12">
      <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="mb-4 opacity-50"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><line x1="10" y1="9" x2="8" y2="9"/></svg>
      <p class="text-lg mb-2">暂无文件，请先选择一个文件夹</p>
      <p class="text-sm opacity-70 text-center max-w-md">选择后，您可以从 Excel 复制一列文本并直接在表格中通过 Ctrl+V 粘贴以实现批量名称映射。</p>
    </div>

    <!-- Footer Action -->
    <div class="shrink-0 mt-6 flex justify-end" v-if="files.length > 0">
      <button 
        @click="executeRename"
        :disabled="isProcessing"
        class="px-8 py-3 bg-foreground text-background font-medium rounded-md hover:bg-neutral-200 transition-colors shadow-lg disabled:opacity-50 disabled:cursor-not-allowed flex items-center focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-background focus:ring-foreground"
      >
        <svg v-if="isProcessing" class="animate-spin -ml-1 mr-2 h-4 w-4 text-background" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        {{ isProcessing ? '执行中...' : '执行重命名' }}
      </button>
    </div>
  </div>
</template>

<style>
/* 避免原生蓝色选中框 */
::selection {
  background-color: #404040;
  color: #f5f5f5;
}
</style>