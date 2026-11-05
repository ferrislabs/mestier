import { formatMoney } from '#/pages/customers/lib/format'
import type { MonthTrend } from '#/pages/treasury/lib/ledger-model'

export type TrendTick = { value: number; ratio: number }

export type TrendBar = {
	month: string
	label: string
	longLabel: string
	receipts: number
	expenses: number
	receiptsRatio: number
	expensesRatio: number
}

export type TrendModel = { top: number; ticks: TrendTick[]; bars: TrendBar[] }

const SHORT = new Intl.DateTimeFormat('fr-FR', {
	month: 'short',
	year: '2-digit',
	timeZone: 'UTC',
})

const LONG = new Intl.DateTimeFormat('fr-FR', {
	month: 'long',
	year: 'numeric',
	timeZone: 'UTC',
})

function monthDate(month: string): Date {
	const [year, index] = month.split('-').map(Number)
	return new Date(Date.UTC(year, index - 1, 1))
}

export function niceCeiling(value: number): number {
	if (value <= 0) return 1
	const magnitude = 10 ** Math.floor(Math.log10(value))
	const normalized = value / magnitude
	const step = [1, 2, 2.5, 5, 10].find((candidate) => normalized <= candidate)
	return Math.ceil((step ?? 10) * magnitude)
}

export function trendModel(trend: readonly MonthTrend[]): TrendModel {
	const highest = Math.max(
		0,
		...trend.flatMap((item) => [item.receipts, item.expenses]),
	)
	const top = niceCeiling(highest)
	return {
		top,
		ticks: [0, top / 2, top].map((value) => ({ value, ratio: value / top })),
		bars: trend.map((item) => ({
			month: item.month,
			label: SHORT.format(monthDate(item.month)),
			longLabel: LONG.format(monthDate(item.month)),
			receipts: item.receipts,
			expenses: item.expenses,
			receiptsRatio: Math.max(0, item.receipts) / top,
			expensesRatio: Math.max(0, item.expenses) / top,
		})),
	}
}

export function trendDescription(trend: readonly MonthTrend[]): string {
	return trend
		.map(
			(item) =>
				`${LONG.format(monthDate(item.month))} : recettes ${formatMoney(item.receipts)}, dépenses ${formatMoney(item.expenses)}.`,
		)
		.join(' ')
}
