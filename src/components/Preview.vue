<template>
  <div class="preview">
    <div class="header">
      <h3>提取预览</h3>
      <el-button-group v-if="store.results.length > 0" size="small">
        <el-button type="success" @click="store.exportExcel">导出 Excel</el-button>
        <el-button type="success" @click="store.exportMarkdown">导出 Markdown</el-button>
      </el-button-group>
    </div>

    <el-table :data="store.results" style="width: 100%" max-height="300" size="small" v-if="store.results.length > 0">
      <el-table-column prop="document_name" label="文档" width="120" show-overflow-tooltip />
      <el-table-column prop="rule_name" label="规则" width="80" />
      <el-table-column prop="content_type" label="类型" width="50">
        <template #default="{ row }">
          <el-tag :type="row.content_type === 'Text' ? '' : 'warning'" size="small">
            {{ row.content_type === 'Text' ? '文本' : '表格' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="content" label="内容" show-overflow-tooltip>
        <template #default="{ row }">
          <div class="content-cell" :title="row.content">
            {{ truncate(row.content, 80) }}
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="position.description" label="位置" width="100" show-overflow-tooltip />
    </el-table>

    <el-empty description="暂无提取结果" v-else :image-size="60" />
  </div>
</template>

<script setup lang="ts">
import { useMainStore } from '../stores/main';

const store = useMainStore();

function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text;
  return text.substring(0, maxLen) + '...';
}
</script>

<style scoped>
.preview {
  padding: 0;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header h3 {
  margin: 0 0 8px 0;
  font-size: 14px;
}

.content-cell {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}
</style>