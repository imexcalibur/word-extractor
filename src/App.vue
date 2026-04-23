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
      <el-row :gutter="20">
        <!-- 左侧：文件列表 -->
        <el-col :span="6">
          <el-card shadow="hover">
            <FileList />
          </el-card>
        </el-col>

        <!-- 中间：规则配置 -->
        <el-col :span="6">
          <el-card shadow="hover">
            <RuleConfig />
          </el-card>
        </el-col>

        <!-- 右侧：预览和导出 -->
        <el-col :span="12">
          <el-card shadow="hover">
            <Preview />
          </el-card>
        </el-col>
      </el-row>

      <!-- 底部：提取按钮 -->
      <el-row class="action-row">
        <el-col :span="24" style="text-align: center;">
          <el-button
            type="primary"
            size="large"
            @click="store.extractContent"
            :loading="store.isLoading"
            :disabled="store.documents.length === 0 || store.rules.length === 0"
          >
            开始提取
          </el-button>
        </el-col>
      </el-row>

      <!-- 错误提示 -->
      <el-alert
        v-if="store.error"
        :title="store.error"
        type="error"
        show-icon
        @close="store.error = null"
        style="margin-top: 16px;"
      />
    </el-main>
  </el-container>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
}

body {
  margin: 0;
  background-color: #f5f7fa;
}

.el-card {
  border-radius: 8px;
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
  padding: 0 20px;
}

.app-header h1 {
  margin: 0;
  font-size: 20px;
}

.app-main {
  padding: 20px;
}

.action-row {
  margin-top: 20px;
}
</style>