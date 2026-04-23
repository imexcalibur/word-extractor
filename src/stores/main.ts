import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
import type { Document, ExtractionRule, ExtractionResult, Template } from '../types';

export const useMainStore = defineStore('main', {
  state: () => ({
    documents: [] as Document[],
    rules: [] as ExtractionRule[],
    results: [] as ExtractionResult[],
    templates: [] as Template[],
    isLoading: false,
    error: null as string | null,
    success: null as string | null,
  }),

  actions: {
    async addFiles() {
      try {
        this.isLoading = true;
        this.error = null;

        const files = await open({
          multiple: true,
          filters: [{ name: 'Word Documents', extensions: ['docx', 'doc'] }],
        });

        if (files) {
          const paths = Array.isArray(files) ? files : [files];
          const newDocs = await invoke<Document[]>('add_files', { paths });
          this.documents.push(...newDocs);
        }
      } catch (e) {
        this.error = String(e);
      } finally {
        this.isLoading = false;
      }
    },

    async removeDocument(docId: string) {
      try {
        await invoke('remove_document', { docId });
        this.documents = this.documents.filter(d => d.id !== docId);
      } catch (e) {
        this.error = String(e);
      }
    },

    async clearDocuments() {
      try {
        await invoke('clear_documents');
        this.documents = [];
      } catch (e) {
        this.error = String(e);
      }
    },

    addRule(rule: ExtractionRule) {
      this.rules.push(rule);
    },

    removeRule(ruleId: string) {
      this.rules = this.rules.filter(r => r.id !== ruleId);
    },

    clearRules() {
      this.rules = [];
    },

    async extractContent() {
      if (this.documents.length === 0 || this.rules.length === 0) {
        this.error = '请先添加文档和提取规则';
        return;
      }

      try {
        this.isLoading = true;
        this.error = null;
        this.results = [];

        const docIds = this.documents.map(d => d.id);
        this.results = await invoke<ExtractionResult[]>('extract_content', {
          docIds,
          rules: this.rules,
        });
        this.success = `提取完成，共 ${this.results.length} 条结果`;
      } catch (e) {
        this.error = String(e);
      } finally {
        this.isLoading = false;
      }
    },

    async exportExcel() {
      if (this.results.length === 0) {
        this.error = '没有可导出的结果';
        return;
      }

      try {
        this.error = null;
        const savePath = await save({
          filters: [{ name: 'Excel', extensions: ['xlsx'] }],
          defaultPath: '提取结果.xlsx',
        });

        if (savePath) {
          await invoke('export_to_excel', {
            results: this.results,
            outputPath: savePath,
          });
          this.success = `已导出到: ${savePath}`;
        }
      } catch (e) {
        this.error = String(e);
      }
    },

    async exportMarkdown() {
      if (this.results.length === 0) {
        this.error = '没有可导出的结果';
        return;
      }

      try {
        this.error = null;
        const savePath = await save({
          filters: [{ name: 'Markdown', extensions: ['md'] }],
          defaultPath: '提取结果.md',
        });

        if (savePath) {
          await invoke('export_to_markdown', {
            results: this.results,
            outputPath: savePath,
          });
          this.success = `已导出到: ${savePath}`;
        }
      } catch (e) {
        this.error = String(e);
      }
    },

    async loadTemplates() {
      try {
        this.templates = await invoke<Template[]>('load_templates');
      } catch (e) {
        this.error = String(e);
      }
    },

    async saveTemplate(name: string) {
      try {
        const template: Template = {
          id: '',
          name,
          rules: this.rules,
        };
        await invoke('save_template', { template });
        await this.loadTemplates();
        this.success = `模板 "${name}" 已保存`;
      } catch (e) {
        this.error = String(e);
      }
    },

    applyTemplate(template: Template) {
      this.rules = [...template.rules];
    },

    clearMessages() {
      this.error = null;
      this.success = null;
    },
  },
});