import { CircleCheck, OctagonAlert, TriangleAlert } from 'lucide-react'
import { cn } from '#/lib/utils'
import type { CollectionOperation } from '#/pages/automation/lib/collections'
import type {
	CollectionDiff,
	VersionChange,
} from '#/pages/automation/lib/openapi-diff'
import { MethodBadge } from '#/pages/automation/ui/method-badge'

const VERSION_LABELS: Record<VersionChange, string> = {
	major: 'Version majeure',
	minor: 'Version mineure',
	patch: 'Version corrective',
	same: 'Même version',
	downgrade: 'Version antérieure',
	different: 'Version différente',
}

type Entry = {
	key: string
	operation: CollectionOperation
	messages: readonly string[]
}

function plural(count: number, singular: string, pluralForm: string) {
	return `${count} ${count > 1 ? pluralForm : singular}`
}

function OperationEntries({
	title,
	entries,
	tone,
}: {
	title: string
	entries: readonly Entry[]
	tone: 'breaking' | 'compatible'
}) {
	if (entries.length === 0) return null
	return (
		<section aria-label={title} className="grid gap-2">
			<h3 className="font-medium text-sm">
				{title}{' '}
				<span className="text-muted-foreground tabular-nums">
					({entries.length})
				</span>
			</h3>
			<ul className="divide-y divide-border overflow-hidden rounded-md border border-border bg-card">
				{entries.map((entry) => (
					<li key={entry.key} className="px-3 py-2">
						<p className="flex min-w-0 items-center gap-2">
							<MethodBadge method={entry.operation.method} />
							<span className="truncate font-mono text-sm">
								{entry.operation.path}
							</span>
							<span className="ml-auto shrink-0 truncate text-muted-foreground text-xs">
								{entry.operation.summary}
							</span>
						</p>
						<ul className="mt-1.5 grid gap-0.5 pl-1 text-sm">
							{entry.messages.map((message) => (
								<li
									key={message}
									className={cn(
										'flex gap-2',
										tone === 'breaking'
											? 'text-state-alert'
											: 'text-muted-foreground',
									)}
								>
									<span aria-hidden>•</span>
									{message}
								</li>
							))}
						</ul>
					</li>
				))}
			</ul>
		</section>
	)
}

export function CollectionDiffView({
	diff,
	warning,
}: {
	diff: CollectionDiff
	warning: string | null
}) {
	const breaking: Entry[] = [
		...diff.removed.map((operation) => ({
			key: `removed-${operation.id}`,
			operation,
			messages: ['Opération supprimée'],
		})),
		...diff.changed.flatMap((entry) => {
			const messages = entry.changes
				.filter((change) => change.severity === 'breaking')
				.map((change) => change.message)
			return messages.length > 0
				? [
						{
							key: `breaking-${entry.next.id}`,
							operation: entry.next,
							messages,
						},
					]
				: []
		}),
	]
	const compatible: Entry[] = [
		...diff.added.map((operation) => ({
			key: `added-${operation.id}`,
			operation,
			messages: ['Nouvelle opération'],
		})),
		...diff.changed.flatMap((entry) => {
			const messages = entry.changes
				.filter((change) => change.severity === 'compatible')
				.map((change) => change.message)
			return messages.length > 0
				? [
						{
							key: `compatible-${entry.next.id}`,
							operation: entry.next,
							messages,
						},
					]
				: []
		}),
	]

	return (
		<div className="grid gap-4">
			<div className="flex flex-wrap items-center gap-x-3 gap-y-1">
				<p className="font-mono text-base">
					{diff.previousVersion}{' '}
					<span className="text-muted-foreground">→</span> {diff.nextVersion}
				</p>
				<span className="rounded-sm border border-border px-1.5 py-0.5 text-muted-foreground text-xs">
					{VERSION_LABELS[diff.versionChange]}
				</span>
				<p className="text-muted-foreground text-sm">
					{plural(diff.added.length, 'ajoutée', 'ajoutées')} ·{' '}
					{plural(diff.removed.length, 'supprimée', 'supprimées')} ·{' '}
					{plural(diff.changed.length, 'modifiée', 'modifiées')}
				</p>
			</div>
			{warning && (
				<p className="flex items-start gap-2 rounded-md border border-state-today/50 bg-state-today/10 px-3 py-2 text-sm">
					<TriangleAlert
						className="mt-0.5 size-4 shrink-0 text-state-today"
						aria-hidden
					/>
					{warning}
				</p>
			)}
			{diff.breakingCount > 0 ? (
				<div
					role="alert"
					className="flex items-start gap-2 rounded-md border border-state-alert/50 bg-state-alert/8 px-3 py-2 text-sm"
				>
					<OctagonAlert
						className="mt-0.5 size-4 shrink-0 text-state-alert"
						aria-hidden
					/>
					<p>
						<span className="font-medium text-state-alert">
							{plural(
								diff.breakingCount,
								'changement incompatible',
								'changements incompatibles',
							)}
						</span>{' '}
						<span className="text-muted-foreground">
							Rien n’est modifié dans vos workflows : après la mise à jour, les
							nœuds concernés sont signalés en rouge jusqu’à ce que vous les
							corrigiez.
						</span>
					</p>
				</div>
			) : (
				!diff.identical && (
					<p className="flex items-start gap-2 rounded-md border border-border bg-muted/40 px-3 py-2 text-sm">
						<CircleCheck
							className="mt-0.5 size-4 shrink-0 text-(color:--auto-success-text)"
							aria-hidden
						/>
						Aucun changement incompatible : les workflows existants continuent
						de fonctionner.
					</p>
				)
			)}
			<OperationEntries
				title="Changements incompatibles"
				entries={breaking}
				tone="breaking"
			/>
			<OperationEntries
				title="Changements compatibles"
				entries={compatible}
				tone="compatible"
			/>
		</div>
	)
}
