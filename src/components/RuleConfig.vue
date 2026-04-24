<template>
  <div class="rule-config">
    <div class="header">
      <h3>提取规则配置</h3>
    </div>

    <el-form label-width="80px" size="small">
      <el-form-item label="规则类型">
        <el-select v-model="ruleType" placeholder="选择规则类型" @change="resetConfig">
          <el-option label="章节标题" value="ChapterTitle" />
          <el-option label="表格关键词" value="TableKeyword" />
          <el-option label="段落关键词" value="ParagraphKeyword" />
        </el-select>
      </el-form-item>

      <el-form-item label="规则名称">
        <el-input v-model="ruleName" placeholder="输入规则名称" />
      </el-form-item>

      <!-- 章节标题配置 -->
      <template v-if="ruleType === 'ChapterTitle'">
        <el-form-item label="标题匹配">
          <el-input v-model="chapterTitlePattern" placeholder="输入标题文本或正则表达式" />
        </el-form-item>
        <el-form-item label="识别方式">
          <el-checkbox-group v-model="chapterDetectModes">
            <el-checkbox label="HeadingStyle">内置标题样式</el-checkbox>
            <el-checkbox label="FormatFeature">格式特征(加粗大字)</el-checkbox>
            <el-checkbox label="NumberPattern">编号模式</el-checkbox>
          </el-checkbox-group>
        </el-form-item>
        <el-form-item label="还原编号">
          <el-switch v-model="chapterRestoreNumbering" />
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
        <el-form-item label="关键词">
          <el-input v-model="tableKeyword" placeholder="输入关键词定位表格内容" />
        </el-form-item>
        <el-form-item label="提取范围">
          <el-select v-model="tableExtractMode">
            <el-option label="单元格内容" value="Cell" />
            <el-option label="单元格全部(保留结构)" value="CellFull" />
            <el-option label="单元格+相邻格" value="CellAdjacent" />
            <el-option label="整行" value="RowFull" />
            <el-option label="整列" value="ColumnFull" />
            <el-option label="到下一个标题行" value="ToNextHeading" />
            <el-option label="整个表格" value="Table" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="tableExtractMode === 'CellAdjacent'" label="相邻方向">
          <el-select v-model="tableAdjacentDirection">
            <el-option label="左侧" value="Left" />
            <el-option label="右侧" value="Right" />
            <el-option label="上方" value="Above" />
            <el-option label="下方" value="Below" />
          </el-select>
        </el-form-item>
        <el-form-item label="还原编号">
          <el-switch v-model="tableRestoreNumbering" />
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
          <el-input v-model="paragraphKeyword" placeholder="输入关键词" />
        </el-form-item>
        <el-form-item label="提取范围">
          <el-select v-model="paragraphRangeMode">
            <el-option label="单段落" value="Single" />
            <el-option label="多段落" value="Multi" />
            <el-option label="到标记为止" value="UntilMarker" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="paragraphRangeMode === 'UntilMarker'" label="结束标记">
          <el-input v-model="paragraphEndMarker" placeholder="输入结束标记" />
        </el-form-item>
        <el-form-item v-if="paragraphRangeMode === 'Multi'" label="最大段落数">
          <el-input-number v-model="paragraphMaxParagraphs" :min="1" :max="20" size="small" />
        </el-form-item>
        <el-form-item label="还原编号">
          <el-switch v-model="paragraphRestoreNumbering" />
        </el-form-item>
      </template>

      <el-form-item>
        <el-button type="primary" size="small" @click="addRule" :disabled="!canAdd">添加规则</el-button>
      </el-form-item>
    </el-form>

    <!-- 已添加规则列表 -->
    <div class="rules-list" v-if="store.rules.length > 0">
      <h4>已添加规则:</h4>
      <el-table :data="store.rules" max-height="150" size="small">
        <el-table-column prop="name" label="名称" show-overflow-tooltip />
        <el-table-column prop="rule_type" label="类型" width="60">
          <template #default="{ row }">
            {{ getTypeLabel(row.rule_type) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="50">
          <template #default="{ row }">
            <el-button type="danger" size="small" link @click="store.removeRule(row.id)">
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
import {
  RuleType, MatchMode, RangeMode, TableExtractMode, HeadingDetectMode, AdjacentDirection,
  type ExtractionRule, type RuleConfig
} from '../types';

const store = useMainStore();

const ruleType = ref<RuleType>(RuleType.ChapterTitle);
const ruleName = ref('');

// 章节配置
const chapterTitlePattern = ref('');
const chapterDetectModes = ref<string[]>(['HeadingStyle', 'NumberPattern']);
const chapterRestoreNumbering = ref(true);
const chapterIncludeSubsections = ref(true);
const chapterIncludeTables = ref(true);

// 表格配置
const tableKeyword = ref('');
const tableExtractMode = ref<TableExtractMode>(TableExtractMode.CellFull);
const tableAdjacentDirection = ref<AdjacentDirection>(AdjacentDirection.Right);
const tableRestoreNumbering = ref(true);

// 段落配置
const paragraphMatchMode = ref<MatchMode>(MatchMode.Prefix);
const paragraphKeyword = ref('');
const paragraphRangeMode = ref<RangeMode>(RangeMode.Single);
const paragraphEndMarker = ref('');
const paragraphMaxParagraphs = ref(5);
const paragraphRestoreNumbering = ref(true);

const canAdd = computed(() => {
  if (!ruleName.value) return false;
  switch (ruleType.value) {
    case RuleType.ChapterTitle:
      return !!chapterTitlePattern.value;
    case RuleType.TableKeyword:
      return !!tableKeyword.value;
    case RuleType.ParagraphKeyword:
      return !!paragraphKeyword.value;
    default:
      return false;
  }
});

function resetConfig() {
  chapterTitlePattern.value = '';
  chapterDetectModes.value = ['HeadingStyle', 'NumberPattern'];
  chapterRestoreNumbering.value = true;
  chapterIncludeSubsections.value = true;
  chapterIncludeTables.value = true;
  tableKeyword.value = '';
  tableExtractMode.value = TableExtractMode.CellFull;
  tableAdjacentDirection.value = AdjacentDirection.Right;
  tableRestoreNumbering.value = true;
  paragraphMatchMode.value = MatchMode.Prefix;
  paragraphKeyword.value = '';
  paragraphRangeMode.value = RangeMode.Single;
  paragraphEndMarker.value = '';
  paragraphMaxParagraphs.value = 5;
  paragraphRestoreNumbering.value = true;
}

function getConfig(): RuleConfig {
  switch (ruleType.value) {
    case RuleType.ChapterTitle:
      return {
        title_pattern: chapterTitlePattern.value,
        heading_detect_modes: chapterDetectModes.value as HeadingDetectMode[],
        restore_numbering: chapterRestoreNumbering.value,
        include_subsections: chapterIncludeSubsections.value,
        include_tables: chapterIncludeTables.value,
      };
    case RuleType.TableKeyword:
      return {
        table_keyword: tableKeyword.value,
        table_extract_mode: tableExtractMode.value,
        adjacent_direction: tableAdjacentDirection.value,
        restore_numbering: tableRestoreNumbering.value,
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
        restore_numbering: paragraphRestoreNumbering.value,
      };
    default:
      return {};
  }
}

function addRule() {
  const config = getConfig();
  const rule: ExtractionRule = {
    id: crypto.randomUUID(),
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
  padding: 0;
}

.header h3 {
  margin: 0 0 8px 0;
  font-size: 14px;
}

.rules-list {
  margin-top: 8px;
}

.rules-list h4 {
  margin: 0 0 4px 0;
  color: #666;
  font-size: 12px;
}

.el-checkbox-group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
</style>