<template>
  <div class="preview">
    <div class="header">
      <h3>📊 提取预览</h3>
      <el-button-group v-if="store.results.length > 0">
        <el-button type="success" @click="store.exportExcel">导出 Excel</el-button>
        <el-button type="success" @click="store.exportMarkdown">导出 Markdown</el-button>
      </el-button-group>
    </div>

    <el-table :data="store.results" style="width: 100%" max-height="400" v-if="store.results.length > 0">
      <el-table-column prop="document_name" label="文档名称" width="150" />
      <el-table-column prop="rule_name" label="规则" width="100" />
      <el-table-column prop="content_type" label="类型" width="80">
        <template #default="{ row }">
          <el-tag :type="row.content_type === 'Text' ? '' : 'warning'" size="small">
            {{ row.content_type === 'Text' ? '文本' : '表格' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="content" label="提取内容">
        <template #default="{ row }">
          <div class="content-cell" :title="row.content">
            {{ truncate(row.content, 100) }}
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="position.description" label="位置" width="120" />
    </el-table>

    <el-empty description="暂无提取结果" v-else />
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
  padding: 16px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header h3 {
  margin: 0;
}

.content-cell {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
}
</style>