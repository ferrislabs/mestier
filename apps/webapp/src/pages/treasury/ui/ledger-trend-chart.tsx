import { useId } from 'react'
import { formatMoney, formatMoneyRounded } from '#/pages/customers/lib/format'
import { trendDescription, trendModel } from '#/pages/treasury/lib/chart-model'
import type { MonthTrend } from '#/pages/treasury/lib/ledger-model'

const WIDTH = 640
const HEIGHT = 230
const LEFT = 64
const RIGHT = 12
const TOP = 12
const BOTTOM = 30
const BAR = 26
const GAP = 4

export function LedgerTrendChart({ trend }: { trend: readonly MonthTrend[] }) {
	const baseId = useId()
	const model = trendModel(trend)
	const plotWidth = WIDTH - LEFT - RIGHT
	const plotHeight = HEIGHT - TOP - BOTTOM
	const slot = plotWidth / Math.max(1, model.bars.length)
	const yOf = (ratio: number) => TOP + plotHeight * (1 - ratio)
	return (
		<figure className="mb-6 rounded-lg border bg-card p-4">
			<figcaption className="mb-2 flex flex-wrap items-center justify-between gap-x-4 gap-y-1">
				<span className="font-medium text-sm">
					Recettes et dépenses des six derniers mois
				</span>
				<span className="flex items-center gap-4 text-[12px] text-muted-foreground">
					<span className="flex items-center gap-1.5">
						<span
							aria-hidden
							className="size-2.5 rounded-sm bg-state-progress"
						/>
						Recettes
					</span>
					<span className="flex items-center gap-1.5">
						<span aria-hidden className="size-2.5 rounded-sm bg-state-today" />
						Dépenses
					</span>
				</span>
			</figcaption>
			<svg
				role="img"
				viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
				aria-label="Graphique en barres des recettes et des dépenses par mois sur les six derniers mois"
				aria-describedby={`${baseId}-desc`}
				className="mx-auto h-auto max-h-64 w-full"
			>
				<title>Recettes et dépenses des six derniers mois</title>
				<desc id={`${baseId}-desc`}>{trendDescription(trend)}</desc>
				{model.ticks.map((tick) => (
					<g key={tick.value}>
						<line
							x1={LEFT}
							x2={WIDTH - RIGHT}
							y1={yOf(tick.ratio)}
							y2={yOf(tick.ratio)}
							className="stroke-border"
							strokeDasharray={tick.value === 0 ? undefined : '3 3'}
						/>
						<text
							x={LEFT - 8}
							y={yOf(tick.ratio)}
							textAnchor="end"
							dominantBaseline="middle"
							className="fill-muted-foreground"
							fontSize="11"
						>
							{formatMoneyRounded(tick.value)}
						</text>
					</g>
				))}
				{model.bars.map((bar, index) => {
					const center = LEFT + slot * index + slot / 2
					const receiptsHeight = plotHeight * bar.receiptsRatio
					const expensesHeight = plotHeight * bar.expensesRatio
					return (
						<g key={bar.month}>
							<rect
								x={center - BAR - GAP / 2}
								y={yOf(bar.receiptsRatio)}
								width={BAR}
								height={receiptsHeight}
								rx={2}
								className="fill-state-progress"
							>
								<title>
									{`Recettes, ${bar.longLabel} : ${formatMoney(bar.receipts)}`}
								</title>
							</rect>
							<rect
								x={center + GAP / 2}
								y={yOf(bar.expensesRatio)}
								width={BAR}
								height={expensesHeight}
								rx={2}
								className="fill-state-today"
							>
								<title>
									{`Dépenses, ${bar.longLabel} : ${formatMoney(bar.expenses)}`}
								</title>
							</rect>
							<text
								x={center}
								y={HEIGHT - 10}
								textAnchor="middle"
								className="fill-muted-foreground"
								fontSize="11"
							>
								{bar.label}
							</text>
						</g>
					)
				})}
			</svg>
		</figure>
	)
}
