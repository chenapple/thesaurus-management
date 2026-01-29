import { ref } from 'vue';
import { driver, type DriveStep, type Driver } from 'driver.js';
import 'driver.js/dist/driver.css';
import * as api from '../api';

// 教程完成状态的存储 key
const ONBOARDING_COMPLETED_KEY = '__onboarding_tour_completed';

export type ViewMode = 'dashboard' | 'keywords' | 'roots' | 'wordcloud' | 'monitoring' | 'smartcopy' | 'knowledge' | 'ads' | 'agent' | 'weekly_report';

export interface OnboardingTourOptions {
  onSwitchView?: (mode: ViewMode) => void;
}

export function useOnboardingTour(options: OnboardingTourOptions = {}) {
  const isRunning = ref(false);
  const driverInstance = ref<Driver | null>(null);

  // 切换视图的辅助函数
  const switchView = (mode: ViewMode) => {
    if (options.onSwitchView) {
      options.onSwitchView(mode);
    }
  };

  // 等待元素出现
  const waitForElement = (selector: string, timeout = 2000): Promise<Element | null> => {
    return new Promise((resolve) => {
      const element = document.querySelector(selector);
      if (element) {
        resolve(element);
        return;
      }

      const observer = new MutationObserver(() => {
        const el = document.querySelector(selector);
        if (el) {
          observer.disconnect();
          resolve(el);
        }
      });

      observer.observe(document.body, {
        childList: true,
        subtree: true,
      });

      setTimeout(() => {
        observer.disconnect();
        resolve(document.querySelector(selector));
      }, timeout);
    });
  };

  // 检查教程是否已完成
  async function isCompleted(): Promise<boolean> {
    try {
      return await api.hasApiKey(ONBOARDING_COMPLETED_KEY);
    } catch (e) {
      console.error('检查教程完成状态失败:', e);
      return false;
    }
  }

  // 标记教程为已完成
  async function markCompleted(): Promise<void> {
    try {
      await api.setApiKey(ONBOARDING_COMPLETED_KEY, 'true');
    } catch (e) {
      console.error('保存教程完成状态失败:', e);
    }
  }

  // 重置教程状态（用于从设置菜单重新启动）
  async function resetTour(): Promise<void> {
    try {
      await api.deleteApiKey(ONBOARDING_COMPLETED_KEY);
    } catch (e) {
      console.error('重置教程状态失败:', e);
    }
  }

  // 定义 14 个教程步骤
  const createTourSteps = (): DriveStep[] => [
    // 步骤 1: 全屏欢迎
    {
      popover: {
        title: '欢迎使用词库管理工具',
        description: '让我们快速了解一下各个功能模块，帮助你更高效地管理亚马逊关键词和优化 Listing。',
        side: 'over',
        align: 'center',
      },
    },
    // 步骤 2: 功能导航区
    {
      element: '.view-toggle',
      popover: {
        title: '功能导航',
        description: '通过这里切换不同的功能模块，包括概览、词库管理、排名监控等。',
        side: 'bottom',
        align: 'center',
      },
    },
    // 步骤 3: 概览按钮
    {
      element: '.view-toggle .el-button:first-child',
      popover: {
        title: '数据概览',
        description: '查看产品数据汇总面板，包括关键词数量统计、词根分析概况等。',
        side: 'bottom',
        align: 'start',
      },
    },
    // 步骤 4: 词库管理按钮 (关键词、词根、词云)
    // 在这一步点击下一步时，切换到 keywords 视图
    {
      element: '.view-toggle .el-button:nth-child(2)',
      popover: {
        title: '词库管理',
        description: '管理产品的关键词数据，支持关键词列表、词根分析、词云可视化三种视图。点击下一步将切换到词库视图。',
        side: 'bottom',
        align: 'start',
        onNextClick: async () => {
          // 先切换视图
          switchView('keywords');
          // 等待产品侧边栏出现
          await waitForElement('.sidebar');
          // 短暂延迟确保 DOM 完全更新
          await new Promise(resolve => setTimeout(resolve, 150));
          // 手动移动到下一步
          driverInstance.value?.moveNext();
        },
      },
    },
    // 步骤 5: 产品列表
    {
      element: '.sidebar',
      popover: {
        title: '产品列表',
        description: '在这里选择要分析的产品，每个产品都有独立的关键词库和分析数据。',
        side: 'right',
        align: 'start',
      },
    },
    // 步骤 6: 添加产品按钮
    {
      element: '.sidebar .sidebar-header .el-button',
      popover: {
        title: '创建新产品',
        description: '点击这里创建新的产品，开始构建你的关键词库。',
        side: 'bottom',
        align: 'start',
      },
    },
    // 步骤 7: 数据菜单
    {
      element: '.header-actions .el-dropdown',
      popover: {
        title: '数据管理',
        description: '通过这里导入关键词 Excel 数据、导出分析结果、管理数据备份。',
        side: 'bottom',
        align: 'end',
      },
    },
    // 步骤 8: 排名监控
    {
      element: '.view-toggle .el-button:nth-child(5)',
      popover: {
        title: '排名监控',
        description: '监控关键词的搜索排名变化，及时了解关键词表现。',
        side: 'bottom',
        align: 'center',
      },
    },
    // 步骤 9: 智能文案
    {
      element: '.view-toggle .el-button:nth-child(6)',
      popover: {
        title: '智能文案',
        description: '使用 AI 生成优化的 Listing 标题、五点描述和产品描述建议。',
        side: 'bottom',
        align: 'center',
      },
    },
    // 步骤 10: 智能广告
    {
      element: '.view-toggle .el-button:nth-child(7)',
      popover: {
        title: '智能广告',
        description: '上传广告报告，AI 分析广告表现并提供优化建议。',
        side: 'bottom',
        align: 'center',
      },
    },
    // 步骤 11: 知识库
    {
      element: '.view-toggle .el-button:nth-child(8)',
      popover: {
        title: '知识库',
        description: '构建企业知识库，上传文档资料，使用 AI 问答检索知识。',
        side: 'bottom',
        align: 'center',
      },
    },
    // 步骤 12: 设置入口
    {
      element: '.global-settings-dropdown',
      popover: {
        title: '系统设置',
        description: '在这里配置 API Key、快捷键等系统设置。',
        side: 'bottom',
        align: 'end',
      },
    },
    // 步骤 13: 帮助按钮
    {
      element: '.nav-help-btn',
      popover: {
        title: '帮助文档',
        description: '遇到问题？点击这里查看详细的帮助文档和使用指南。',
        side: 'bottom',
        align: 'end',
      },
    },
    // 步骤 14: 全屏完成
    {
      popover: {
        title: '教程完成！',
        description: '你已经了解了所有主要功能。现在可以开始使用了！如需再次查看教程，可以在「设置」菜单中找到「新手教程」选项。',
        side: 'over',
        align: 'center',
        onNextClick: async () => {
          // 最后切换回概览页面
          switchView('dashboard');
          // 标记完成
          await markCompleted();
          // 销毁教程
          driverInstance.value?.destroy();
        },
      },
    },
  ];

  // 启动教程
  function startTour() {
    if (isRunning.value) return;

    isRunning.value = true;

    const tourSteps = createTourSteps();

    driverInstance.value = driver({
      showProgress: true,
      steps: tourSteps,
      nextBtnText: '下一步',
      prevBtnText: '上一步',
      doneBtnText: '完成',
      progressText: '{{current}} / {{total}}',
      allowClose: true,
      stagePadding: 8,
      stageRadius: 8,
      popoverClass: 'onboarding-popover',
      onDestroyStarted: async () => {
        if (driverInstance.value) {
          driverInstance.value.destroy();
        }
        isRunning.value = false;
      },
      onDestroyed: () => {
        isRunning.value = false;
        driverInstance.value = null;
      },
    });

    driverInstance.value.drive();
  }

  // 停止教程
  function stopTour() {
    if (driverInstance.value) {
      driverInstance.value.destroy();
    }
    isRunning.value = false;
  }

  // 从设置菜单重新启动教程
  async function restartTour() {
    await resetTour();
    startTour();
  }

  return {
    isRunning,
    isCompleted,
    startTour,
    stopTour,
    restartTour,
    markCompleted,
    resetTour,
  };
}
