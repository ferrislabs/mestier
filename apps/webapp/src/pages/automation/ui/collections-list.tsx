import { Link } from '@tanstack/react-router'
import { Ellipsis, Library, Trash2, Upload } from 'lucide-react'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import { collectionPath } from '#/pages/automation/lib/automation-paths'
import {
	describeSource,
	formatImportDate,
	operationCount,
} from '#/pages/automation/lib/collection-format'
import type { Collection } from '#/pages/automation/lib/collections'

export function CollectionsList({
	organizationSlug,
	collections,
	onImport,
	onRemove,
}: {
	organizationSlug: string
	collections: readonly Collection[]
	onImport: () => void
	onRemove: (collection: Collection) => void
}) {
	return (
		<PageShell className="gap-0">
			<PageHeader
				className="mb-6"
				title="Collections"
				description="Les specs OpenAPI importées. Leurs opérations sont disponibles comme nœuds dans tous les workflows."
				actions={
					<Button onClick={onImport}>
						<Upload />
						Importer une collection
					</Button>
				}
			/>
			{collections.length === 0 ? (
				<div className="rounded-lg border border-border border-dashed bg-card px-6 py-12 text-center">
					<p className="font-medium">Aucune collection pour le moment.</p>
					<p className="mt-1 text-muted-foreground text-sm">
						Importez la spec OpenAPI d’un service externe pour l’appeler depuis
						vos workflows sans écrire les requêtes à la main.
					</p>
				</div>
			) : (
				<ul className="divide-y divide-border overflow-hidden rounded-lg border border-border bg-card">
					{collections.map((collection) => (
						<li
							key={collection.id}
							className="relative flex items-center gap-4 px-4 py-3 transition-colors hover:bg-muted/50"
						>
							<Library
								className="size-5 shrink-0 text-muted-foreground"
								strokeWidth={1.6}
								aria-hidden
							/>
							<div className="min-w-0 flex-1">
								<div className="flex items-center gap-2">
									<Link
										to={buildOrgPath(
											organizationSlug,
											collectionPath(collection.id),
										)}
										className="truncate font-medium text-base outline-none after:absolute after:inset-0 focus-visible:after:ring-2 focus-visible:after:ring-ring focus-visible:after:ring-inset"
									>
										{collection.title}
									</Link>
									<Badge variant="secondary" className="font-mono">
										v{collection.version}
									</Badge>
								</div>
								<p className="mt-0.5 truncate text-muted-foreground text-sm">
									{describeSource(collection.source)}, le{' '}
									{formatImportDate(collection.importedAt)}
								</p>
							</div>
							<div className="hidden w-64 min-w-0 text-right md:block">
								<p className="truncate font-mono text-muted-foreground text-xs">
									{collection.baseUrl}
								</p>
								<p className="mt-0.5 text-sm">
									{operationCount(collection.operations.length)}
								</p>
							</div>
							<DropdownMenu>
								<DropdownMenuTrigger asChild>
									<Button
										variant="ghost"
										size="icon-sm"
										className="relative z-10"
										aria-label={`Actions sur ${collection.title}`}
									>
										<Ellipsis />
									</Button>
								</DropdownMenuTrigger>
								<DropdownMenuContent align="end">
									<DropdownMenuItem
										variant="destructive"
										onSelect={() => onRemove(collection)}
									>
										<Trash2 />
										Supprimer la collection
									</DropdownMenuItem>
								</DropdownMenuContent>
							</DropdownMenu>
						</li>
					))}
				</ul>
			)}
		</PageShell>
	)
}
