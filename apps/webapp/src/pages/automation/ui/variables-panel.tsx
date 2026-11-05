import {
	Check,
	ChevronLeft,
	ChevronRight,
	FlaskConical,
	LoaderCircle,
} from 'lucide-react'
import { type DragEvent, useEffect, useState } from 'react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { formatItemCount } from '#/pages/automation/lib/format'
import type { UpstreamEntry } from '#/pages/automation/lib/upstream-entries'
import {
	listVariables,
	type Variable,
	variableExpression,
} from '#/pages/automation/lib/variables'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'

export interface TestRun {
	running: boolean
	start: () => void
}

const DRAGGING_FLAG = 'draggingExpression'

function distanceLabel(entry: UpstreamEntry): string {
	if (entry.direct)
		return entry.branch
			? `entrée directe, branche ${entry.branch}`
			: 'entrée directe'
	return `${entry.distance} nœuds avant`
}

function startDrag(event: DragEvent, expression: string) {
	event.dataTransfer.setData('text/plain', expression)
	event.dataTransfer.effectAllowed = 'copy'
	document.documentElement.dataset[DRAGGING_FLAG] = 'true'
}

function endDrag() {
	delete document.documentElement.dataset[DRAGGING_FLAG]
}

function ValuePlaceholder({ variable }: { variable: Variable }) {
	return (
		<span
			aria-hidden
			className="block h-2 rounded-full bg-muted-foreground/15"
			style={{ width: `${40 + ((variable.key.length * 13) % 45)}%` }}
		/>
	)
}

function VariableRow({
	variable,
	expression,
	copied,
	placeholder,
	onCopy,
}: {
	variable: Variable
	expression: string
	copied: boolean
	placeholder: boolean
	onCopy: () => void
}) {
	return (
		<li>
			<button
				type="button"
				draggable
				onDragStart={(event) => startDrag(event, expression)}
				onDragEnd={endDrag}
				onClick={onCopy}
				aria-label={`${variable.key}, ${variable.type}. Glisser dans un champ ou cliquer pour copier ${expression}`}
				className="grid w-full cursor-grab grid-cols-[minmax(0,1fr)_4.5rem_minmax(0,1fr)] items-center gap-2 rounded-sm px-2 py-1 text-left text-xs outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring active:cursor-grabbing"
			>
				<span
					className="truncate font-mono text-foreground"
					style={{ paddingLeft: `${variable.depth * 12}px` }}
				>
					{variable.key}
				</span>
				<span className="text-muted-foreground">{variable.type}</span>
				<span className="truncate text-muted-foreground">
					{copied ? (
						<span className="inline-flex items-center gap-1 text-foreground">
							<Check className="size-3" aria-hidden />
							Copiée
						</span>
					) : placeholder && variable.type !== 'objet' ? (
						<ValuePlaceholder variable={variable} />
					) : placeholder ? null : (
						variable.preview
					)}
				</span>
			</button>
		</li>
	)
}

export type InputPanelMode = 'variables' | 'execution'

function runSummary(entry: UpstreamEntry, notRunLabel: string): string {
	if (!entry.status) return notRunLabel
	if (entry.status === 'error') return 'échec'
	return formatItemCount(entry.items.length)
}

function DataNotice({ testRun }: { testRun: TestRun }) {
	return (
		<div className="grid justify-items-start gap-2 rounded-md border border-border border-dashed bg-card px-3 py-3">
			<p className="font-medium text-sm">Aucune exécution pour le moment</p>
			<p className="text-muted-foreground text-xs">
				Les données réelles des nœuds précédents s’affichent ici après une
				exécution du workflow.
			</p>
			<Button
				size="sm"
				variant="outline"
				onClick={testRun.start}
				disabled={testRun.running}
			>
				{testRun.running ? (
					<LoaderCircle className="animate-spin" />
				) : (
					<FlaskConical />
				)}
				{testRun.running ? 'Exécution en cours…' : 'Tester le workflow'}
			</Button>
		</div>
	)
}

function ItemPager({
	index,
	count,
	onChange,
}: {
	index: number
	count: number
	onChange: (index: number) => void
}) {
	return (
		<div className="flex items-center justify-between gap-2 px-2 pt-1 pb-1.5 text-muted-foreground text-xs">
			<span className="tabular-nums">
				Élément {index + 1} sur {count}
			</span>
			<span className="flex gap-0.5">
				<Button
					variant="ghost"
					size="icon-sm"
					className="size-6"
					aria-label="Élément précédent"
					disabled={index === 0}
					onClick={() => onChange(index - 1)}
				>
					<ChevronLeft />
				</Button>
				<Button
					variant="ghost"
					size="icon-sm"
					className="size-6"
					aria-label="Élément suivant"
					disabled={index >= count - 1}
					onClick={() => onChange(index + 1)}
				>
					<ChevronRight />
				</Button>
			</span>
		</div>
	)
}

function EmptyEntry({ text, alert }: { text: string; alert?: boolean }) {
	return (
		<p
			className={cn(
				'px-2 py-2 text-muted-foreground text-xs',
				alert && 'text-state-alert',
			)}
		>
			{text}
		</p>
	)
}

export function VariablesPanel({
	upstream,
	mode,
	testRun,
	intro,
	notRunLabel = () => 'pas encore exécuté',
}: {
	upstream: readonly UpstreamEntry[]
	mode: InputPanelMode
	testRun?: TestRun
	intro?: string
	notRunLabel?: (entry: UpstreamEntry) => string
}) {
	const [copiedExpression, setCopiedExpression] = useState<string | null>(null)
	const [itemIndexes, setItemIndexes] = useState<Record<string, number>>({})

	useEffect(() => {
		if (!copiedExpression) return
		const timer = window.setTimeout(() => setCopiedExpression(null), 1500)
		return () => window.clearTimeout(timer)
	}, [copiedExpression])

	const copy = (expression: string) => {
		navigator.clipboard?.writeText(expression).catch(() => undefined)
		setCopiedExpression(expression)
	}

	const showData = mode === 'execution'
	const executedCount = upstream.filter((entry) => entry.status).length

	const renderRows = (
		entry: UpstreamEntry,
		item: unknown,
		placeholder = false,
	) => {
		const variables = item ? listVariables(item) : []
		if (variables.length === 0)
			return <EmptyEntry text="Ce nœud ne produit aucune donnée connue." />
		return (
			<ul>
				{variables.map((variable) => {
					const expression = variableExpression(entry.rootPath, variable.path)
					return (
						<VariableRow
							key={expression}
							variable={variable}
							expression={expression}
							copied={copiedExpression === expression}
							placeholder={placeholder}
							onCopy={() => copy(expression)}
						/>
					)
				})}
			</ul>
		)
	}

	const renderData = (entry: UpstreamEntry) => {
		if (!entry.status) return renderRows(entry, entry.items[0], true)
		if (entry.status === 'error')
			return (
				<EmptyEntry
					alert
					text={`Échec lors de l’exécution : ${entry.error ?? 'erreur inconnue'}.`}
				/>
			)
		if (entry.items.length === 0)
			return (
				<EmptyEntry text="Aucun élément transmis vers ce nœud lors de l’exécution." />
			)
		const index = Math.min(itemIndexes[entry.id] ?? 0, entry.items.length - 1)
		return (
			<>
				{entry.items.length > 1 && (
					<ItemPager
						index={index}
						count={entry.items.length}
						onChange={(next) =>
							setItemIndexes((current) => ({
								...current,
								[entry.id]: next,
							}))
						}
					/>
				)}
				{renderRows(entry, entry.items[index])}
			</>
		)
	}

	return (
		<div className="grid gap-3">
			{showData && executedCount === 0 && testRun ? (
				<DataNotice testRun={testRun} />
			) : (
				<p className="text-muted-foreground text-sm">
					{intro ??
						`${showData ? 'Données réelles de la dernière exécution.' : 'Clés disponibles dans les nœuds précédents.'} Glissez une clé dans un champ des paramètres pour l’utiliser. Un clic copie son expression.`}
				</p>
			)}
			{upstream.map((entry) => {
				return (
					<details
						key={entry.id}
						open
						className="group rounded-md border border-border bg-card"
					>
						<summary className="flex cursor-pointer list-none items-center gap-2 px-2.5 py-2 outline-none focus-visible:ring-2 focus-visible:ring-ring [&::-webkit-details-marker]:hidden">
							<ChevronRight
								className="size-4 shrink-0 text-muted-foreground transition-transform group-open:rotate-90"
								aria-hidden
							/>
							<KindIconTile icon={entry.icon} hue={entry.hue} size="sm" />
							<span className="min-w-0 flex-1">
								<span className="block truncate font-medium text-sm">
									{entry.label}
								</span>
								<span className="block text-muted-foreground text-xs">
									{distanceLabel(entry)}
									{showData
										? ` · ${runSummary(entry, notRunLabel(entry))}`
										: entry.executed
											? ''
											: ', données d’exemple'}
								</span>
							</span>
						</summary>
						<div className="border-border border-t px-1 py-1">
							{showData ? renderData(entry) : renderRows(entry, entry.items[0])}
						</div>
					</details>
				)
			})}
		</div>
	)
}
