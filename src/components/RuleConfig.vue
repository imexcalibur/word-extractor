<template>
  <div class="rule-config">
    <div class="header">
      <h3>📋 提取规则配置</h3>
    </div>

    <el-form label-width="80px" style="margin-top: 16px">
      <el-form-item label="规则类型">
        <el-select v-model="ruleType" placeholder="选择规则类型" @change="resetConfig">
          <el-option label="📖 章节标题定位" value="ChapterTitle" />
          <el-option label="📊 表格关键词定位" value="TableKeyword" />
          <el-option label="📝 段落关键词定位" value="ParagraphKeyword" />
        </el-select>
      </el-form-item>

      <el-form-item label="规则名称">
        <el-input v-model="ruleName" placeholder="输入规则名称" />
      </el-form-item>

      <!-- 章节标题配置 -->
      <template v-if="ruleType === 'ChapterTitle'">
        <el-form-item label="标题匹配">
          <el-input v-model="chapterTitlePattern" placeholder="如: 第.*章.*风险" />
        </el-form-item>
        <el-form-item label="包含子章节">
          <el-switch v-model="chapterIncludeSubsections" />
        </el-form-item>
        <el-form-item label="包含表格">
          <el-switch v-model="chapterIncludeTables" />
        </el-form-item>
      </template>

      <!-- 表格关键词配置 -->
      <template v-if="ruleType === 'TableKeyword'">
        <el-form-item label="匹配列头">
          <el-input v-model="tableColumnHeader" placeholder="如: 风险等级" />
        </el-form-item>
        <el-form-item label="匹配值">
          <el-input v-model="tableColumnValue" placeholder="如: 高 (可选)" />
        </el-form-item>
        <el-form-item label="提取范围">
          <el-select v-model="tableExtractMode">
            <el-option label="单元格" value="Cell" />
            <el-option label="整行" value="Row" />
            <el-option label="整个表格" value="Table" />
          </el-select>
        </el-form-item>
      </template>

      <!-- 段落关键词配置 -->
      <template v-if="ruleType === 'ParagraphKeyword'">
        <el-form-item label="匹配模式">
          <el-select v-model="paragraphMatchMode">
            <el-option label="前缀匹配" value="Prefix" />
            <el-option label="包含匹配" value="Contains" />
            <el-option label="正则匹配" value="Regex" />
          </el-select>
        </el-form-item>
        <el-form-item label="关键词">
          <el-input v-model="paragraphKeyword" placeholder="如: 整改措施：" />
        </el-form-item>
        <el-form-item label="提取范围">
          <el-select v-model="paragraphRangeMode">
            <el-option label="单段落" value="Single" />
            <el-option label="多段落" value="Multi" />
            <el-option label="到标记为止" value="UntilMarker" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="paragraphRangeMode === 'UntilMarker'" label="结束标记">
          <el-input v-model="paragraphEndMarker" placeholder="如: 责任人：" />
        </el-form-item>
        <el-form-item v-if="paragraphRangeMode === 'Multi'" label="最大段落数">
          <el-input-number v-model="paragraphMaxParagraphs" :min="1" :max="20" />
        </el-form-item>
      </template>

      <el-form-item>
        <el-button type="primary" @click="addRule" :disabled="!canAdd">添加规则</el-button>
      </el-form-item>
    </el-form>

    <!-- 已添加规则列表 -->
    <div class="rules-list" v-if="store.rules.length > 0">
      <h4>已添加规则:</h4>
      <el-table :data="store.rules" max-height="200">
        <el-table-column prop="name" label="名称" />
        <el-table-column prop="rule_type" label="类型" width="100">
          <template #default="{ row }">
            {{ getTypeLabel(row.rule_type) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="80">
          <template #default="{ row }">
            <el-button type="danger" size="small" @click="store.removeRule(row.id)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useMainStore } from '../stores/main';
import { RuleType, MatchMode, RangeMode, ExtractMode, type ExtractionRule, type RuleConfig } from '../types';

const store = useMainStore();

const ruleType = ref<RuleType>(RuleType.ChapterTitle);
const ruleName = ref('');

// 章节配置 - 单独变量
const chapterTitlePattern = ref('');
const chapterIncludeSubsections = ref(true);
const chapterIncludeTables = ref(true);

// 表格配置 - 单独变量
const tableColumnHeader = ref('');
const tableColumnValue = ref('');
const tableExtractMode = ref<ExtractMode>(ExtractMode.Row);

// 段落配置 - 单独变量
const paragraphMatchMode = ref<MatchMode>(MatchMode.Prefix);
const paragraphKeyword = ref('');
const paragraphRangeMode = ref<RangeMode>(RangeMode.Single);
const paragraphEndMarker = ref('');
const paragraphMaxParagraphs = ref(5);

const canAdd = computed(() => {
  if (!ruleName.value) return false;
  switch (ruleType.value) {
    case RuleType.ChapterTitle:
      return !!chapterTitlePattern.value;
    case RuleType.TableKeyword:
      return !!tableColumnHeader.value;
    case RuleType.ParagraphKeyword:
      return !!paragraphKeyword.value;
    default:
      return false;
  }
});

function resetConfig() {
  chapterTitlePattern.value = '';
  chapterIncludeSubsections.value = true;
  chapterIncludeTables.value = true;
  tableColumnHeader.value = '';
  tableColumnValue.value = '';
  tableExtractMode.value = ExtractMode.Row;
  paragraphMatchMode.value = MatchMode.Prefix;
  paragraphKeyword.value = '';
  paragraphRangeMode.value = RangeMode.Single;
  paragraphEndMarker.value = '';
  paragraphMaxParagraphs.value = 5;
}

function getConfig(): RuleConfig {
  switch (ruleType.value) {
    case RuleType.ChapterTitle:
      return {
        title_pattern: chapterTitlePattern.value,
        include_subsections: chapterIncludeSubsections.value,
        include_tables: chapterIncludeTables.value,
      };
    case RuleType.TableKeyword:
      return {
        column_match: {
          header: tableColumnHeader.value,
          value: tableColumnValue.value,
        },
        extract_mode: tableExtractMode.value,
      };
    case RuleType.ParagraphKeyword:
      return {
        paragraph_match: {
          match_mode: paragraphMatchMode.value,
          keyword: paragraphKeyword.value,
        },
        extract_range: {
          mode: paragraphRangeMode.value,
          end_marker: paragraphEndMarker.value,
          max_paragraphs: paragraphMaxParagraphs.value,
        },
      };
    default:
      return {};
  }
}

function addRule() {
  const config = getConfig();
  const rule: ExtractionRule = {
    id: '',
    name: ruleName.value,
    rule_type: ruleType.value,
    config,
  };
  store.addRule(rule);
  ruleName.value = '';
  resetConfig();
}

function getTypeLabel(type: RuleType): string {
  switch (type) {
    case RuleType.ChapterTitle: return '章节';
    case RuleType.TableKeyword: return '表格';
    case RuleType.ParagraphKeyword: return '段落';
    default: return '';
  }
}
</script>

<style scoped>
.rule-config {
  padding: 16px;
}

.header h3 {
  margin: 0;
}

.rules-list {
  margin-top: 16px;
}

.rules-list h4 {
  margin: 0 0 8px 0;
  color: #666;
}
</style>