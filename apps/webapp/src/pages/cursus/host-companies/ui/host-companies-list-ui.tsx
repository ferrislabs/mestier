import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, StatusBadge } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { cn } from '#/lib/utils'
import type { HostCompanyValues } from '#/pages/cursus/host-companies/lib/host-companies'
import { CreateHostCompanyDialog } from '#/pages/cursus/host-companies/ui/create-host-company-dialog'
import { FormError } from '#/pages/cursus/participants/ui/field-input'
import {
	archiveQualifier,
	countSummary,
	HOST_COMPANY_NOUN,
} from '#/pages/cursus/shared/lib/count-summary'
import {
	type ArchiveFilter,
	FEMININE_ARCHIVE_OPTIONS,
} from '#/pages/cursus/shared/lib/list-filters'
import { ListEmptyRow } from '#/pages/cursus/shared/ui/list-empty'
import { ListToolbar } from '#/pages/cursus/shared/ui/list-toolbar'

export interface HostCompanyRowVM {
	id: string
	name: string
	siret: string
	nafCode: string
	city: string
	employeeCount: string
	archived: boolean
}

export interface HostCompaniesListUIProps {
	rows: HostCompanyRowVM[]
	total: number | undefined
	search: string
	isLoading: boolean
	error: string | null
	archived: ArchiveFilter
	canManage: boolean
	createOpen: boolean
	isCreating: boolean
	createError: string | null
	onSearchChange: (search: string) => void
	onArchivedChange: (archived: ArchiveFilter) => void
	onResetFilters: () => void
	onCreateOpenChange: (open: boolean) => void
	onCreate: (values: HostCompanyValues) => void
	onOpen: (hostCompanyId: string) => void
}

export function HostCompaniesListUI({
	rows,
	total,
	search,
	isLoading,
	error,
	archived,
	canManage,
	createOpen,
	isCreating,
	createError,
	onSearchChange,
	onArchivedChange,
	onResetFilters,
	onCreateOpenChange,
	onCreate,
	onOpen,
}: HostCompaniesListUIProps) {
	const filtered = search.trim() !== '' || archived !== 'active'

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title="Entreprises d'accueil"
				description={
					total === undefined
						? undefined
						: countSummary(total, HOST_COMPANY_NOUN, archiveQualifier(archived))
				}
				actions={
					canManage ? (
						<Button
							type="button"
							size="sm"
							onClick={() => onCreateOpenChange(true)}
						>
							<Plus aria-hidden /> Nouvelle entreprise
						</Button>
					) : null
				}
			/>
			<ListToolbar
				search={search}
				searchPlaceholder="Rechercher un nom, un SIRET"
				onSearch={onSearchChange}
				status={archived}
				statusOptions={FEMININE_ARCHIVE_OPTIONS}
				statusLabel="Filtrer par statut"
				onStatus={onArchivedChange}
			/>
			<FormError message={error} />
			<div className="overflow-hidden rounded-lg border bg-card">
				<Table>
					<TableHeader>
						<TableRow className="hover:bg-transparent">
							<TableHead className="pl-4">Nom de l'entreprise</TableHead>
							<TableHead>Numéro de SIRET</TableHead>
							<TableHead>NAF</TableHead>
							<TableHead>Ville</TableHead>
							<TableHead>Employés</TableHead>
							<TableHead className="pr-4">Statut</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{isLoading || error ? null : rows.length === 0 ? (
							<ListEmptyRow
								colSpan={6}
								title="Aucune entreprise ne correspond."
								hint={
									filtered
										? 'Vérifie l’orthographe ou le statut sélectionné.'
										: undefined
								}
								onReset={filtered ? onResetFilters : undefined}
							/>
						) : (
							rows.map((row) => (
								<TableRow
									key={row.id}
									className={cn(
										'relative cursor-pointer focus-within:bg-muted/50',
										row.archived && 'opacity-60',
									)}
								>
									<TableCell className="py-2.5 pl-4">
										<button
											type="button"
											className="font-medium outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
											onClick={() => onOpen(row.id)}
										>
											{row.name}
										</button>
									</TableCell>
									<TableCell className="tabular">{row.siret}</TableCell>
									<TableCell>{row.nafCode}</TableCell>
									<TableCell>{row.city}</TableCell>
									<TableCell className="tabular">{row.employeeCount}</TableCell>
									<TableCell className="pr-4">
										<StatusBadge tone={row.archived ? 'neutral' : 'success'}>
											{row.archived ? 'Archivée' : 'Active'}
										</StatusBadge>
									</TableCell>
								</TableRow>
							))
						)}
					</TableBody>
				</Table>
			</div>

			{canManage ? (
				<CreateHostCompanyDialog
					open={createOpen}
					isPending={isCreating}
					error={createError}
					onOpenChange={onCreateOpenChange}
					onSubmit={onCreate}
				/>
			) : null}
		</PageShell>
	)
}
