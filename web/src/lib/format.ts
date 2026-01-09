/**
 * Truncates an Ethereum address to show first 6 and last 4 characters.
 * Example: 0x1234567890abcdef... -> 0x1234...cdef
 */
export function truncateAddress(address: string): string {
	if (address.length <= 13) return address;
	return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

/**
 * Formats a date string as relative time (e.g., "Just now", "5m ago", "Yesterday").
 */
export function formatRelativeDate(dateString: string): string {
	const date = new Date(dateString);
	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

	if (diffDays === 0) {
		const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
		if (diffHours === 0) {
			const diffMins = Math.floor(diffMs / (1000 * 60));
			return diffMins <= 1 ? "Just now" : `${diffMins}m ago`;
		}
		return `${diffHours}h ago`;
	}
	if (diffDays === 1) return "Yesterday";
	if (diffDays < 7) return `${diffDays}d ago`;

	return date.toLocaleDateString("en-US", {
		month: "short",
		day: "numeric",
	});
}

/**
 * Formats a date string as absolute date (e.g., "Jan 10, 2026").
 */
export function formatDate(dateString: string): string {
	const date = new Date(dateString);
	return date.toLocaleDateString("en-US", {
		year: "numeric",
		month: "short",
		day: "numeric",
	});
}

/**
 * Formats a date string with time (e.g., "Jan 10, 2026, 02:30 PM").
 */
export function formatDateTime(dateString: string): string {
	const date = new Date(dateString);
	return date.toLocaleDateString("en-US", {
		year: "numeric",
		month: "short",
		day: "numeric",
		hour: "2-digit",
		minute: "2-digit",
	});
}
