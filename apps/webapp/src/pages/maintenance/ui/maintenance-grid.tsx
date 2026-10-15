import { cn } from '#/lib/utils'
import { describeCadence } from '#/pages/maintenance/lib/cadence-form'
import {
	buildLineCells,
	contractTitle,
	type GridCell,
	seasonLabel,
	sheetWeeks,
} from '#/pages/maintenance/lib/grid-model'
import { weekLabel } from '#/pages/maintenance/lib/weeks'
import type { Grid, GridContract, GridLine } from '#/pages/maintenance/types'
import { CELL_STYLES, STATE_LABELS } from '#/pages/maintenance/ui/cell-styles'
import type { VisitEntryTarget } from '#/pages/maintenance/ui/visit-entry-dialog'

export interface MaintenanceGridProps {
	grid: Grid
	/** The week to highlight, or `null` when the sheet shows another year. */
	currentWeek: number | null
	onCellClick: (target: VisitEntryTarget) => void
}

const CELL = 'h-9 min-w-9 w-9 p-0 text-center align-middle text-xs'
const STICKY = 'sticky left-0 z-[1] bg-card'

/**
 * The paper sheet: a block per contract, a row per line, a column per week,
 * the day of the month in the cell. Pure presentation; the dates in it were
 * all decided by the backend.
 */
export function MaintenanceGrid({
	grid,
	currentWeek,
	onCellClick,
}: MaintenanceGridProps) {
	const weeks = sheetWeeks(grid.contracts)

	return (
		<div className="overflow-x-auto border bg-card">
			<table className="border-separate border-spacing-0 text-sm">
				<thead>
					<tr>
						<th
							scope="col"
							className={cn(
								STICKY,
								'z-[2] min-w-56 border-b px-3 py-2 text-left font-medium',
							)}
						>
							Prestation
						</th>
						{weeks.map((week) => (
							<th
								key={week}
								scope="col"
								aria-current={week === currentWeek ? 'date' : undefined}
								className={cn(
									CELL,
									'border-b font-medium text-muted-foreground',
									week === currentWeek && 'bg-brand-soft text-primary',
								)}
							>
								{weekLabel(week)}
							</th>
						))}
					</tr>
				</thead>
				{grid.contracts.map((contract) => (
					<ContractBlock
						key={contract.id}
						contract={contract}
						weeks={weeks}
						currentWeek={currentWeek}
						onCellClick={onCellClick}
					/>
				))}
			</table>
		</div>
	)
}

interface ContractBlockProps {
	contract: GridContract
	weeks: number[]
	currentWeek: number | null
	onCellClick: (target: VisitEntryTarget) => void
}

function ContractBlock({
	contract,
	weeks,
	currentWeek,
	onCellClick,
}: ContractBlockProps) {
	return (
		<tbody>
			<tr>
				<th
					scope="colgroup"
					colSpan={weeks.length + 1}
					className="border-b bg-muted/50 p-0 text-left font-normal"
				>
					<div className="sticky left-0 flex w-fit flex-wrap items-baseline gap-x-3 px-3 py-2">
						<span className="font-semibold">{contractTitle(contract)}</span>
						<span className="text-xs text-muted-foreground">
							{[contract.site.city, seasonLabel(contract.season)]
								.filter(Boolean)
								.join(' - ')}
							{contract.status === 'DRAFT' ? ' - brouillon' : ''}
						</span>
					</div>
				</th>
			</tr>
			{contract.lines.map((line) => (
				<LineRow
					key={line.id}
					contract={contract}
					line={line}
					weeks={weeks}
					currentWeek={currentWeek}
					onCellClick={onCellClick}
				/>
			))}
		</tbody>
	)
}

interface LineRowProps extends ContractBlockProps {
	line: GridLine
}

function LineRow({
	contract,
	line,
	weeks,
	currentWeek,
	onCellClick,
}: LineRowProps) {
	const cells = buildLineCells(contract, line, weeks)

	return (
		<tr>
			<th
				scope="row"
				className={cn(STICKY, 'border-b px-3 py-1.5 text-left font-normal')}
			>
				<div className="font-medium">{line.label}</div>
				<div className="text-xs text-muted-foreground">
					{describeCadence(line.cadence)}
					{line.minutes_per_intervention
						? ` - ${line.minutes_per_intervention} min`
						: ''}
				</div>
			</th>
			{cells.map((cell) => (
				<SheetCell
					key={cell.week}
					cell={cell}
					isCurrentWeek={cell.week === currentWeek}
					onClick={() =>
						onCellClick({
							lineId: line.id,
							lineLabel: line.label,
							customerName: contract.site.customer_name,
							week: cell.week,
							year: contract.season.year,
						})
					}
					lineLabel={line.label}
					customerName={contract.site.customer_name}
				/>
			))}
		</tr>
	)
}

interface SheetCellProps {
	cell: GridCell
	isCurrentWeek: boolean
	lineLabel: string
	customerName: string
	onClick: () => void
}

function cellName(cell: GridCell): string {
	const label = weekLabel(cell.week)
	const days = cell.visits.map((visit) => visit.day).join(', ')
	if (cell.visits.length > 0)
		return `${label}, ${STATE_LABELS[cell.state]} le ${days}`
	if (cell.due)
		return `${label}, ${STATE_LABELS[cell.state]} le ${cell.due.day}`

	return `${label}, ${STATE_LABELS[cell.state]}`
}

function cellText(cell: GridCell): string {
	if (cell.visits.length > 0) return cell.visits.map((v) => v.day).join('/')
	if (cell.due) return String(cell.due.day)

	return ''
}

function SheetCell({
	cell,
	isCurrentWeek,
	lineLabel,
	customerName,
	onClick,
}: SheetCellProps) {
	return (
		<td
			aria-label={cellName(cell)}
			data-week={cell.week}
			data-state={cell.state}
			className={cn(
				CELL,
				'border-b',
				cell.inWindow && cell.state === 'empty' && 'bg-brand-soft/60',
				isCurrentWeek && cell.state === 'empty' && 'bg-brand-soft',
			)}
		>
			{cell.clickable ? (
				<button
					type="button"
					aria-label={`Saisir une intervention : ${lineLabel}, ${customerName}, ${weekLabel(cell.week)}`}
					title={cell.due ? `Échéance le ${cell.due.day}` : undefined}
					onClick={onClick}
					className={cn(
						'flex h-9 w-full items-center justify-center hover:bg-brand-soft focus-visible:outline-2 focus-visible:outline-ring',
						CELL_STYLES[cell.state],
					)}
				>
					{cellText(cell)}
				</button>
			) : (
				<span
					className={cn(
						'flex h-9 w-full items-center justify-center',
						CELL_STYLES[cell.state],
					)}
				>
					{cellText(cell)}
				</span>
			)}
		</td>
	)
}
