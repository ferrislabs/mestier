import {
	Ellipsis,
	FileUp,
	Loader2,
	Pencil,
	Plus,
	Table2,
	Trash2,
} from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import type { KnowledgeTable } from '#/pages/knowledge-tables/lib/types'
import { Page, PageHeader } from '#/pages/projects/ui/page'

interface TablesListPageProps {
	tables: readonly KnowledgeTable[]
	canManage: boolean
	isLoading: boolean
	error: string | null
	onCreate: () => void
	onImport: () => void
	onOpen: (table: KnowledgeTable) => void
	onEdit: (table: KnowledgeTable) => void
	onDelete: (table: KnowledgeTable) => void
}

function formatDate(iso: string): string {
	return new Date(iso).toLocaleDateString('fr-FR')
}

export function TablesListPage({
	tables,
	canManage,
	isLoading,
	error,
	onCreate,
	onImport,
	onOpen,
	onEdit,
	onDelete,
}: TablesListPageProps) {
	return (
		<Page>
			<PageHeader
				title="Tables"
				description="Les tables de l’entreprise : suivi de chantiers, inventaire, contacts fournisseurs. Des colonnes typées, des vues Grille et Kanban."
				actions={
					canManage ? (
						<div className="flex flex-wrap gap-2">
							<Button type="button" variant="outline" onClick={onImport}>
								<FileUp aria-hidden />
								Créer depuis un CSV
							</Button>
							<Button type="button" onClick={onCreate}>
								<Plus aria-hidden />
								Nouvelle table
							</Button>
						</div>
					) : null
				}
			/>
			{error ? (
				<div
					role="alert"
					className="mb-4 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-destructive text-sm"
				>
					{error}
				</div>
			) : null}
			{isLoading ? (
				<div className="flex items-center justify-center gap-2 py-10 text-muted-foreground text-sm">
					<Loader2 aria-hidden className="size-4 animate-spin" />
					Chargement…
				</div>
			) : tables.length === 0 ? (
				<EmptyState
					title="Aucune table"
					hint={
						canManage
							? 'Crée une première table pour suivre tes chantiers, ton matériel ou tes fournisseurs.'
							: 'Personne n’a encore créé de table.'
					}
				/>
			) : (
				<ul className="divide-y rounded-lg border bg-card">
					{tables.map((table) => (
						<li key={table.id} className="flex items-center gap-2 px-4">
							<button
								type="button"
								className="flex min-w-0 flex-1 items-start gap-3 py-3 text-left outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
								onClick={() => onOpen(table)}
							>
								<Table2
									aria-hidden
									className="mt-0.5 size-4 shrink-0 text-muted-foreground"
								/>
								<span className="grid min-w-0 flex-1 gap-0.5">
									<span className="truncate font-medium">{table.name}</span>
									{table.description ? (
										<span className="line-clamp-1 text-[13px] text-muted-foreground">
											{table.description}
										</span>
									) : null}
								</span>
								<span className="shrink-0 text-muted-foreground text-xs">
									Modifiée le {formatDate(table.updated_at)}
								</span>
							</button>
							{canManage ? (
								<DropdownMenu>
									<DropdownMenuTrigger asChild>
										<Button
											type="button"
											variant="ghost"
											size="icon-sm"
											aria-label={`Actions de la table ${table.name}`}
										>
											<Ellipsis aria-hidden />
										</Button>
									</DropdownMenuTrigger>
									<DropdownMenuContent align="end">
										<DropdownMenuItem onSelect={() => onEdit(table)}>
											<Pencil aria-hidden /> Renommer
										</DropdownMenuItem>
										<DropdownMenuSeparator />
										<DropdownMenuItem
											variant="destructive"
											onSelect={() => onDelete(table)}
										>
											<Trash2 aria-hidden /> Supprimer
										</DropdownMenuItem>
									</DropdownMenuContent>
								</DropdownMenu>
							) : null}
						</li>
					))}
				</ul>
			)}
		</Page>
	)
}
