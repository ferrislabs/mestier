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
import {
	KIND_COPY,
	type ParticipantKind,
	type ParticipantStatus,
	STATUS_LABELS,
} from '#/pages/cursus/participants/lib/participants'
import type { HostCompanyOption } from '#/pages/cursus/participants/ui/create-participant-dialog'
import { FormError } from '#/pages/cursus/participants/ui/field-input'
import {
	APPRENTICE_NOUN,
	type CountNoun,
	countSummary,
	TUTOR_NOUN,
} from '#/pages/cursus/shared/lib/count-summary'
import type { FilterOption } from '#/pages/cursus/shared/lib/list-filters'
import { ListEmpty } from '#/pages/cursus/shared/ui/list-empty'
import { ListToolbar } from '#/pages/cursus/shared/ui/list-toolbar'

export interface ParticipantRowVM {
	id: string
	name: string
	email: string
	hostCompanyNames: string[]
	jobTitles: string[]
	status: ParticipantStatus
}

export interface ParticipantsListUIProps {
	kind: ParticipantKind
	rows: ParticipantRowVM[]
	total: number | undefined
	search: string
	isLoading: boolean
	error: string | null
	status: ParticipantStatus | undefined
	hostCompanyId: string | undefined
	hostCompanies: HostCompanyOption[]
	canManage: boolean
	page: number
	pageCount: number
	onSearchChange: (search: string) => void
	onResetFilters: () => void
	onStatusChange: (status: ParticipantStatus | undefined) => void
	onHostCompanyChange: (hostCompanyId: string | undefined) => void
	onPageChange: (page: number) => void
	onCreate: () => void
	onOpen: (participantId: string) => void
}

type StatusFilter = ParticipantStatus | 'all'

const ALL_COMPANIES = 'all'

const STATUS_OPTIONS: FilterOption<StatusFilter>[] = [
	{ value: 'all', label: 'Tous' },
	{ value: 'pending', label: 'En attente' },
	{ value: 'active', label: 'Actifs' },
	{ value: 'inactive', label: 'Inactifs' },
]

const KIND_NOUN: Record<ParticipantKind, CountNoun> = {
	apprentice: APPRENTICE_NOUN,
	company_tutor: TUTOR_NOUN,
}

const STATUS_TONE: Record<
	ParticipantStatus,
	'success' | 'warning' | 'neutral'
> = {
	active: 'success',
	pending: 'warning',
	inactive: 'neutral',
}

function joined(values: string[]): string {
	return values.length === 0 ? '—' : values.join(', ')
}

export function ParticipantsListUI({
	kind,
	rows,
	total,
	search,
	isLoading,
	error,
	status,
	hostCompanyId,
	hostCompanies,
	canManage,
	page,
	pageCount,
	onSearchChange,
	onResetFilters,
	onStatusChange,
	onHostCompanyChange,
	onPageChange,
	onCreate,
	onOpen,
}: ParticipantsListUIProps) {
	const copy = KIND_COPY[kind]
	const isTutor = kind === 'company_tutor'
	const filtered =
		search.trim() !== '' || status !== undefined || hostCompanyId !== undefined

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title={copy.title}
				description={
					total === undefined
						? undefined
						: countSummary(total, KIND_NOUN[kind], status)
				}
				actions={
					canManage ? (
						<Button type="button" size="sm" onClick={onCreate}>
							<Plus aria-hidden /> {copy.createLabel}
						</Button>
					) : null
				}
			/>
			<ListToolbar
				search={search}
				searchPlaceholder="Rechercher un nom, un e-mail"
				onSearch={onSearchChange}
				status={status ?? 'all'}
				statusOptions={STATUS_OPTIONS}
				statusLabel="Filtrer par statut"
				onStatus={(next) => onStatusChange(next === 'all' ? undefined : next)}
				select={
					isTutor
						? {
								value: hostCompanyId ?? ALL_COMPANIES,
								label: 'Filtrer par entreprise',
								options: [
									{ value: ALL_COMPANIES, label: 'Toutes les entreprises' },
									...hostCompanies.map((company) => ({
										value: company.id,
										label: company.name,
									})),
								],
								onChange: (next) =>
									onHostCompanyChange(
										next === ALL_COMPANIES ? undefined : next,
									),
							}
						: undefined
				}
			/>
			<FormError message={error} />
			{isLoading || error ? null : rows.length === 0 ? (
				<ListEmpty
					title={copy.empty}
					hint={filtered ? copy.emptyHint : undefined}
					onReset={filtered ? onResetFilters : undefined}
				/>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Nom</TableHead>
								{isTutor ? (
									<>
										<TableHead>Entreprise</TableHead>
										<TableHead>Fonction</TableHead>
									</>
								) : (
									<TableHead>E-mail</TableHead>
								)}
								<TableHead className="pr-4">Statut</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{rows.map((row) => (
								<TableRow
									key={row.id}
									className="relative cursor-pointer focus-within:bg-muted/50"
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
									{isTutor ? (
										<>
											<TableCell>{joined(row.hostCompanyNames)}</TableCell>
											<TableCell className="text-muted-foreground">
												{joined(row.jobTitles)}
											</TableCell>
										</>
									) : (
										<TableCell className="text-muted-foreground">
											{row.email}
										</TableCell>
									)}
									<TableCell className="pr-4">
										<StatusBadge tone={STATUS_TONE[row.status]}>
											{STATUS_LABELS[row.status]}
										</StatusBadge>
									</TableCell>
								</TableRow>
							))}
						</TableBody>
					</Table>
				</div>
			)}

			{pageCount > 1 ? (
				<div className="mt-3 flex items-center justify-end gap-3 text-sm">
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={page <= 1}
						onClick={() => onPageChange(page - 1)}
					>
						Précédent
					</Button>
					<span>
						Page {page} sur {pageCount}
					</span>
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={page >= pageCount}
						onClick={() => onPageChange(page + 1)}
					>
						Suivant
					</Button>
				</div>
			) : null}
		</PageShell>
	)
}
