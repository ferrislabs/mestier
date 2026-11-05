import { Link } from '@tanstack/react-router'
import {
	ChevronDown,
	ChevronRight,
	CircleCheck,
	OctagonAlert,
	RefreshCw,
	Upload,
} from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	describeSource,
	formatImportDate,
	operationCount,
} from '#/pages/automation/lib/collection-format'
import type {
	Collection,
	CollectionOperation,
} from '#/pages/automation/lib/collections'
import {
	exampleOf,
	operationContractSource,
} from '#/pages/automation/lib/schema-typescript'
import { MethodBadge } from '#/pages/automation/ui/method-badge'
import { OperationContractTabs } from '#/pages/automation/ui/operation-contract-tabs'

function groupByTag(operations: readonly CollectionOperation[]) {
	const groups = new Map<string, CollectionOperation[]>()
	for (const operation of operations) {
		groups.set(operation.tag, [...(groups.get(operation.tag) ?? []), operation])
	}
	return [...groups.entries()]
}

export interface CollectionUpdateOutcome {
	version: string
	breakingCount: number
}

export function CollectionDetail({
	organizationSlug,
	collection,
	outcome,
	onUpdate,
}: {
	organizationSlug: string
	collection: Collection | undefined
	outcome: CollectionUpdateOutcome | null
	onUpdate: (autoCheck: boolean) => void
}) {
	const [openOperationId, setOpenOperationId] = useState<string | null>(null)
	const listPath = buildOrgPath(organizationSlug, '/automatisation/collections')

	if (!collection) {
		return (
			<PageShell className="gap-0">
				<PageHeader
					className="mb-6"
					title="Collection introuvable"
					description="Cette collection n’existe pas ou a été supprimée."
					actions={
						<Button asChild variant="outline">
							<Link to={listPath}>Voir les collections</Link>
						</Button>
					}
				/>
			</PageShell>
		)
	}

	return (
		<PageShell className="gap-0">
			<nav aria-label="Emplacement de la collection" className="mb-2">
				<ol className="flex items-center gap-1 text-muted-foreground text-sm">
					<li>
						<Link
							to={listPath}
							className="rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
						>
							Collections
						</Link>
					</li>
					<ChevronRight className="size-3.5" aria-hidden />
					<li aria-current="page" className="text-foreground">
						{collection.title}
					</li>
				</ol>
			</nav>
			<PageHeader
				className="mb-6"
				title={collection.title}
				description={collection.description || undefined}
				actions={
					<div className="flex items-center gap-2">
						<Button variant="outline" onClick={() => onUpdate(false)}>
							<Upload />
							Importer une version
						</Button>
						{collection.source.kind === 'url' && (
							<Button onClick={() => onUpdate(true)}>
								<RefreshCw />
								Vérifier les mises à jour
							</Button>
						)}
					</div>
				}
			/>
			{outcome && (
				<output
					className={cn(
						'mb-4 flex items-start gap-2 rounded-md border px-3 py-2 text-sm',
						outcome.breakingCount > 0
							? 'border-state-alert/50 bg-state-alert/8'
							: 'border-border bg-muted/40',
					)}
				>
					{outcome.breakingCount > 0 ? (
						<OctagonAlert
							className="mt-0.5 size-4 shrink-0 text-state-alert"
							aria-hidden
						/>
					) : (
						<CircleCheck
							className="mt-0.5 size-4 shrink-0 text-(color:--auto-success-text)"
							aria-hidden
						/>
					)}
					<span>
						Collection mise à jour vers la version {outcome.version}.
						{outcome.breakingCount > 0
							? ` ${outcome.breakingCount > 1 ? `${outcome.breakingCount} changements incompatibles` : '1 changement incompatible'} : les nœuds concernés sont signalés en rouge dans leurs workflows.`
							: ' Aucun changement incompatible.'}
					</span>
				</output>
			)}
			<dl className="mb-8 grid gap-x-8 gap-y-3 rounded-lg border border-border bg-card p-4 text-sm sm:grid-cols-2">
				<div>
					<dt className="text-muted-foreground">URL de base</dt>
					<dd className="mt-0.5 truncate font-mono">
						{collection.baseUrl || 'Non précisée'}
					</dd>
				</div>
				<div>
					<dt className="text-muted-foreground">Version</dt>
					<dd className="mt-0.5">
						<span className="font-mono">{collection.version}</span>
						{collection.history && collection.history.length > 0 && (
							<span className="text-muted-foreground">
								{' '}
								· précédentes :{' '}
								{collection.history.map((entry, index) => (
									<span
										key={`${entry.version}-${entry.importedAt}`}
										title={`${describeSource(entry.source)}, le ${formatImportDate(entry.importedAt)}`}
									>
										{index > 0 && ', '}
										<span className="font-mono">{entry.version}</span>
									</span>
								))}
							</span>
						)}
					</dd>
				</div>
				<div>
					<dt className="text-muted-foreground">Authentification</dt>
					<dd className="mt-0.5">
						{collection.security.length > 0
							? collection.security.join(', ')
							: 'Aucune'}
					</dd>
				</div>
				<div>
					<dt className="text-muted-foreground">Source</dt>
					<dd className="mt-0.5 truncate">
						{describeSource(collection.source)}, le{' '}
						{formatImportDate(collection.importedAt)}
					</dd>
				</div>
			</dl>
			<h2 className="mb-3 font-semibold text-base">
				{operationCount(collection.operations.length)}
			</h2>
			<div className="grid gap-6">
				{groupByTag(collection.operations).map(([tag, operations]) => (
					<section key={tag} aria-label={tag}>
						<h3 className="mb-2 font-medium text-muted-foreground text-sm">
							{tag}
						</h3>
						<ul className="divide-y divide-border overflow-hidden rounded-lg border border-border bg-card">
							{operations.map((operation) => {
								const open = openOperationId === operation.id
								const Chevron = open ? ChevronDown : ChevronRight
								return (
									<li key={operation.id}>
										<button
											type="button"
											aria-expanded={open}
											onClick={() =>
												setOpenOperationId(open ? null : operation.id)
											}
											className={cn(
												'flex w-full items-center gap-3 px-4 py-3 text-left outline-none transition-colors hover:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset',
												open && 'bg-muted/40',
											)}
										>
											<MethodBadge method={operation.method} />
											<code className="min-w-0 truncate font-mono text-sm">
												{operation.path}
											</code>
											<span className="ml-auto truncate text-muted-foreground text-sm">
												{operation.summary}
											</span>
											<Chevron
												className="size-4 shrink-0 text-muted-foreground"
												aria-hidden
											/>
										</button>
										{open && (
											<div className="border-border border-t bg-muted/20 px-4 py-4">
												<OperationContractTabs
													source={operationContractSource(
														collection,
														operation,
													)}
													example={
														operation.response
															? exampleOf(
																	operation.response,
																	collection.schemas,
																)
															: null
													}
												/>
											</div>
										)}
									</li>
								)
							})}
						</ul>
					</section>
				))}
			</div>
		</PageShell>
	)
}
