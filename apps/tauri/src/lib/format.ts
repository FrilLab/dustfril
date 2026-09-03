import type { Recommendation, RiskLevel } from '../types/workflow';

const numberFormatter = new Intl.NumberFormat('en-US');

export function formatBytes(bytes: number) {
  if (bytes === 0) {
    return '0 B';
  }

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** exponent;

  return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}

export function formatAge(days: number | null) {
  if (days === null) {
    return 'Unknown';
  }

  return `${numberFormatter.format(days)} day(s)`;
}

export function formatDate(timestamp: number | null) {
  if (timestamp === null) {
    return 'Unknown';
  }

  return new Date(timestamp).toLocaleString();
}

export function formatCount(value: number) {
  return numberFormatter.format(value);
}

export function recommendationTone(recommendation: Recommendation) {
  switch (recommendation) {
    case 'Keep':
      return 'border-emerald-400/25 bg-emerald-400/10 text-emerald-100';
    case 'NeedsReview':
      return 'border-amber-400/25 bg-amber-400/10 text-amber-100';
    case 'SafeToClean':
      return 'border-cyan-400/25 bg-cyan-400/10 text-cyan-100';
  }
}

export function riskTone(level: RiskLevel) {
  switch (level) {
    case 'High':
      return 'border-rose-400/30 bg-rose-400/10 text-rose-100';
    case 'Critical':
      return 'border-red-500/40 bg-red-500/15 text-red-50';
    case 'Medium':
      return 'border-amber-400/30 bg-amber-400/10 text-amber-100';
    case 'Low':
      return 'border-emerald-400/30 bg-emerald-400/10 text-emerald-100';
    case 'None':
      return 'border-slate-400/20 bg-slate-400/10 text-slate-200';
  }
}
