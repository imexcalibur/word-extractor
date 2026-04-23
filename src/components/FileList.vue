<template>
  <div class="file-list">
    <div class="header">
      <h3>文件列表</h3>
      <el-button-group size="small">
        <el-button type="primary" @click="store.addFiles" :loading="store.isLoading">
          添加文件
        </el-button>
        <el-button type="danger" @click="store.clearDocuments" :disabled="store.documents.length === 0">
          清空
        </el-button>
      </el-button-group>
    </div>

    <el-table :data="store.documents" style="width: 100%" max-height="200" size="small">
      <el-table-column prop="file_name" label="文件名" show-overflow-tooltip />
      <el-table-column prop="file_size" label="大小" width="70">
        <template #default="{ row }">
          {{ formatSize(row.file_size) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="50">
        <template #default="{ row }">
          <el-button type="danger" size="small" link @click="store.removeDocument(row.id)">
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="footer" v-if="store.documents.length > 0">
      已选择: {{ store.documents.length }} 个文件
    </div>
  </div>
</template>

<script setup lang="ts">
import { useMainStore } from '../stores/main';

const store = useMainStore();

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / 1024 / 1024).toFixed(1) + ' MB';
}
</script>

<style scoped>
.file-list {
  padding: 0;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.header h3 {
  margin: 0;
  font-size: 14px;
}

.footer {
  padding: 4px 0;
  color: #666;
  font-size: 12px;
}
</style>