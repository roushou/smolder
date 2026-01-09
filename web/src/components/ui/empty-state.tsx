interface EmptyStateProps {
	icon: React.ReactNode;
	title: string;
	description: string;
	action?: React.ReactNode;
}

export function EmptyState({
	icon,
	title,
	description,
	action,
}: EmptyStateProps) {
	return (
		<div className="rounded-xl border border-border bg-bg-elevated p-12 text-center">
			<div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-bg-muted">
				{icon}
			</div>
			<h3 className="mb-2 font-medium text-lg text-text">{title}</h3>
			<p className="mb-6 text-sm text-text-secondary">{description}</p>
			{action}
		</div>
	);
}
