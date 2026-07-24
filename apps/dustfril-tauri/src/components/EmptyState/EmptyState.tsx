type EmptyStateProps = {
  message: string;
  compact?: boolean;
};

export function EmptyState(props: EmptyStateProps) {
  return (
    <div
      className={`flex items-center justify-center px-6 text-center text-sm text-slate-500 ${
        props.compact ? 'min-h-[180px]' : 'min-h-[420px]'
      }`}
    >
      {props.message}
    </div>
  );
}
