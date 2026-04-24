<script setup lang="ts">
import { useMainStore } from './stores/main';
import FileList from './components/FileList.vue';
import RuleConfig from './components/RuleConfig.vue';
import Preview from './components/Preview.vue';

const store = useMainStore();
</script>

<template>
  <el-container class="app-container">
    <el-header class="app-header">
      <h1>Word文档内容提取工具</h1>
    </el-header>

    <el-main class="app-main">
      <!-- 第一排：文件列表和规则配置并排 -->
      <el-row :gutter="12">
        <el-col :span="12">
          <el-card shadow="hover" class="compact-card">
            <FileList />
          </el-card>
        </el-col>
        <el-col :span="12">
          <el-card shadow="hover" class="compact-card">
            <RuleConfig />
          </el-card>
        </el-col>
      </el-row>

      <!-- 第二排：预览 -->
      <el-row :gutter="12" style="margin-top: 12px;">
        <el-col :span="24">
          <el-card shadow="hover" class="compact-card">
            <Preview />
          </el-card>
        </el-col>
      </el-row>

      <!-- 底部：提取按钮 -->
      <el-row class="action-row">
        <el-col :span="24" style="text-align: center;">
          <el-button
            type="primary"
            size="default"
            @click="store.extractContent"
            :loading="store.isLoading"
            :disabled="store.documents.length === 0 || store.rules.length === 0"
          >
            开始提取
          </el-button>
        </el-col>
      </el-row>

      <!-- 成功提示 -->
      <el-alert
        v-if="store.success"
        :title="store.success"
        type="success"
        show-icon
        closable
        @close="store.clearMessages()"
        style="margin-top: 12px;"
      />

      <!-- 错误提示 -->
      <el-alert
        v-if="store.error"
        :title="store.error"
        type="error"
        show-icon
        closable
        @close="store.clearMessages()"
        style="margin-top: 12px;"
      />
    </el-main>

    <!-- 版权标记 -->
    <el-footer class="app-footer">
      <span class="copyright">© 2026 hy</span>
    </el-footer>
  </el-container>
</template>

<style>
:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 13px;
}

body {
  margin: 0;
  background-color: #f5f7fa;
}

.el-card {
  border-radius: 6px;
}

.el-card__body {
  padding: 12px !important;
}

.el-form-item {
  margin-bottom: 12px !important;
}

.el-form-item__label {
  font-size: 13px !important;
}

.el-table {
  font-size: 12px !important;
}

.el-button {
  font-size: 13px !important;
}

.el-input__inner {
  font-size: 13px !important;
}

.compact-card .el-card__body {
  padding: 10px !important;
}
</style>

<style scoped>
.app-container {
  min-height: 100vh;
}

.app-header {
  background-color: #409eff;
  color: white;
  display: flex;
  align-items: center;
  padding: 0 16px;
  height: 40px !important;
}

.app-header h1 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.app-main {
  padding: 12px;
}

.action-row {
  margin-top: 12px;
}

.app-footer {
  height: 24px !important;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #f5f7fa;
}

.copyright {
  color: #c0c4cc;
  font-size: 11px;
}
</style>