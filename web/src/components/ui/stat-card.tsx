interface StatCardProps {
	label: string;
	value: number;
	loading?: boolean;
}

export function StatCard({ label, value, loading = false }: StatCardProps) {
	return (
		<div className="rounded-xl border border-border bg-bg-elevated p-5">
			<p className="mb-1 text-sm text-text-muted">{label}</p>
			{loading ? (
				<div className="h-8 w-16 animate-pulse rounded bg-bg-muted" />
			) : (
				<p className="font-semibold text-2xl text-text">{value}</p>
			)}
		</div>
	);
}
