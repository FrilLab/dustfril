import type { Recommendation, RiskLevel } from '../../../types/workflow';

export type BrowserPane = 'scan' | 'analysis' | 'cleanup' | 'audit';

export type BrowserItemKind = 'folder' | 'document' | 'warning' | 'safe';

export type BrowserItem = {
  id: string;
  title: string;
  subtitle: string;
  meta: string;
  badge: string;
  accent: string;
  kind: BrowserItemKind;
  detailLines: string[];
  path?: string;
};

export type PaneConfig = {
  key: BrowserPane;
  title: string;
  description: string;
  count: number;
  accent: string;
};

export type WorkspaceSummary = {
  keepCount: number;
  reviewCount: number;
  safeCount: number;
  reviewBytes: number;
  safeBytes: number;
};

export type StatusMetric = {
  label: string;
  value: string;
};

export type TotalsMetric = {
  label: string;
  value: string;
};

export type AccentRecommendation = Recommendation;
export type AccentRiskLevel = RiskLevel;
