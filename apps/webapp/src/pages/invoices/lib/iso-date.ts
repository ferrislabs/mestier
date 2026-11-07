const ISO_DAY = /^(\d{4})-(\d{2})-(\d{2})$/

function parts(value: string): [number, number, number] | null {
	const match = ISO_DAY.exec(value)
	if (!match) return null
	const year = Number(match[1])
	const month = Number(match[2])
	const day = Number(match[3])
	const date = new Date(Date.UTC(year, month - 1, day))
	const valid =
		date.getUTCFullYear() === year &&
		date.getUTCMonth() === month - 1 &&
		date.getUTCDate() === day
	return valid ? [year, month, day] : null
}

export function isIsoDate(value: string): boolean {
	return parts(value) !== null
}

export function addDaysIso(value: string, days: number): string {
	const found = parts(value)
	if (!found) return value
	const [year, month, day] = found
	return new Date(Date.UTC(year, month - 1, day + days))
		.toISOString()
		.slice(0, 10)
}

export function daysBetweenIso(from: string, to: string): number | null {
	const start = parts(from)
	const end = parts(to)
	if (!start || !end) return null
	return Math.round(
		(Date.UTC(end[0], end[1] - 1, end[2]) -
			Date.UTC(start[0], start[1] - 1, start[2])) /
			86_400_000,
	)
}
