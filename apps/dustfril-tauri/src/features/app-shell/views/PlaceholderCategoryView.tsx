import { EmptyState } from '../../../components/EmptyState/EmptyState';
import type { CategoryConfig } from '../../../model/categories';

type PlaceholderCategoryViewProps = {
  category: CategoryConfig;
};

export function PlaceholderCategoryView(props: PlaceholderCategoryViewProps) {
  return (
    <div className="space-y-4">
      <section className="rounded-[24px] border border-white/8 bg-[#2b2b2e] px-4 py-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Future Category</p>
        <h2 className="mt-1 text-2xl font-semibold text-white">{props.category.title}</h2>
        <p className="mt-2 max-w-2xl text-sm leading-6 text-slate-300">
          {props.category.description}. This category is reserved for upcoming system management
          features and keeps the sidebar structure ready for expansion.
        </p>
      </section>

      <section className="rounded-[24px] border border-white/8 bg-black/12 p-4">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Planned Items</p>
        <ul className="mt-3 space-y-2 text-sm text-slate-300">
          {(props.category.futureItems ?? []).map((item) => (
            <li key={item} className="rounded-2xl border border-white/8 bg-white/4 px-3 py-2">
              {item}
            </li>
          ))}
        </ul>
      </section>

      <EmptyState message="Scanning and cleanup for this category will be available in a future release." />
    </div>
  );
}
