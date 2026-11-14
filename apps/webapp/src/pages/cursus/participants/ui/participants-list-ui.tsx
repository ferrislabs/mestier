import { Plus } from 'lucide-react'
import type { ReactNode } from 'react'
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
	COMPANY_TUTOR_NOUN,
	type CountNoun,
	countSummary,
} from '#/pages/cursus/shared/lib/count-summary'
import type { FilterOption } from '#/pages/cursus/shared/lib/list-filters'
import { ListEmptyRow } from '#/pages/cursus/shared/ui/list-empty'
import { ListPagination } from '#/pages/cursus/shared/ui/list-pagination'
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
	status: readonly ParticipantStatus[]
	hostCompanyId: string | undefined
	hostCompanies: HostCompanyOption[]
	canManage: boolean
	page: number
	pageCount: number
	onSearchChange: (search: string) => void
	onResetFilters: () => void
	onStatusChange: (status: ParticipantStatus[]) => void
	onHostCompanyChange: (hostCompanyId: string | undefined) => void
	onPageChange: (page: number) => void
	onCreate: () => void
	onOpen: (participantId: string) => void
	header?: ReactNode
	toolbarExtra?: ReactNode
}

const ALL_COMPANIES = 'all'

const STATUS_OPTIONS: FilterOption<ParticipantStatus>[] = [
	{ value: 'pending', label: 'En attente' },
	{ value: 'active', label: 'Actifs' },
	{ value: 'inactive', label: 'Inactifs' },
]

const KIND_NOUN: Record<ParticipantKind, CountNoun> = {
	apprentice: APPRENTICE_NOUN,
	company_tutor: COMPANY_TUTOR_NOUN,
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
	header,
	toolbarExtra,
}: ParticipantsListUIProps) {
	const copy = KIND_COPY[kind]
	const isTutor = kind === 'company_tutor'
	const filtered =
		search.trim() !== '' || status.length > 0 || hostCompanyId !== undefined

	return (
		<PageShell className="block">
			{header ?? (
				<PageHeader
					className="mb-6"
					title={copy.title}
					description={
						total === undefined
							? undefined
							: countSummary(
									total,
									KIND_NOUN[kind],
									status.length === 1 ? status[0] : undefined,
								)
					}
					actions={
						canManage ? (
							<Button type="button" size="sm" onClick={onCreate}>
								<Plus aria-hidden /> {copy.createLabel}
							</Button>
						) : null
					}
				/>
			)}
			<ListToolbar
				extra={toolbarExtra}
				search={search}
				searchPlaceholder="Rechercher un nom, un e-mail"
				onSearch={onSearchChange}
				status={status}
				statusOptions={STATUS_OPTIONS}
				statusLabel="Statut"
				onStatus={onStatusChange}
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
						{isLoading || error ? null : rows.length === 0 ? (
							<ListEmptyRow
								colSpan={isTutor ? 4 : 3}
								title={copy.empty}
								hint={filtered ? copy.emptyHint : undefined}
								onReset={filtered ? onResetFilters : undefined}
							/>
						) : (
							rows.map((row) => (
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
							))
						)}
					</TableBody>
				</Table>
			</div>

			<ListPagination
				page={page}
				pageCount={pageCount}
				onPageChange={onPageChange}
			/>
		</PageShell>
	)
}
