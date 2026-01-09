interface CardSkeletonProps {
	count?: number;
}

export function CardSkeleton({ count = 3 }: CardSkeletonProps) {
	return (
		<div className="space-y-3">
			{Array.from({ length: count }, (_, i) => `skeleton-${i}`).map((key) => (
				<div
					key={key}
					className="animate-pulse rounded-xl border border-border bg-bg-elevated p-5"
				>
					<div className="flex items-center gap-4">
						<div className="h-10 w-10 rounded-lg bg-bg-muted" />
						<div className="flex-1">
							<div className="mb-2 h-4 w-32 rounded bg-bg-muted" />
							<div className="h-3 w-48 rounded bg-bg-muted" />
						</div>
					</div>
				</div>
			))}
		</div>
	);
}
