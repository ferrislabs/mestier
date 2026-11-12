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
	KIND_LABELS,
	type ParticipantKind,
	type ParticipantStatus,
	STATUS_LABELS,
} from '#/pages/cursus/participants/lib/participants'
import { FormError } from '#/pages/cursus/participants/ui/field-input'
import {
	APPRENTICE_NOUN,
	COMPANY_TUTOR_NOUN,
	type CountNoun,
	countSummary,
	PARTICIPANT_NOUN,
} from '#/pages/cursus/shared/lib/count-summary'
import type { FilterOption } from '#/pages/cursus/shared/lib/list-filters'
import { ListEmpty } from '#/pages/cursus/shared/ui/list-empty'
import { ListToolbar } from '#/pages/cursus/shared/ui/list-toolbar'

export interface ParticipantRowVM {
	id: string
	name: string
	email: string
	kinds: ParticipantKind[]
	hostCompanyNames: string[]
	status: ParticipantStatus
}

export interface ParticipantsListUIProps {
	rows: ParticipantRowVM[]
	total: number | undefined
	search: string
	isLoading: boolean
	error: string | null
	kind: ParticipantKind | undefined
	status: ParticipantStatus | undefined
	canManage: boolean
	page: number
	pageCount: number
	onSearchChange: (search: string) => void
	onResetFilters: () => void
	onKindChange: (kind: ParticipantKind | undefined) => void
	onStatusChange: (status: ParticipantStatus | undefined) => void
	onPageChange: (page: number) => void
	onCreate: () => void
	onOpen: (participantId: string) => void
}

type StatusFilter = ParticipantStatus | 'all'
type KindFilter = ParticipantKind | 'all'

const STATUS_OPTIONS: FilterOption<StatusFilter>[] = [
	{ value: 'all', label: 'Tous' },
	{ value: 'pending', label: 'En attente' },
	{ value: 'active', label: 'Actifs' },
	{ value: 'inactive', label: 'Inactifs' },
]

const KIND_OPTIONS: FilterOption<KindFilter>[] = [
	{ value: 'all', label: 'Tous les types' },
	{ value: 'apprentice', label: 'Apprentis' },
	{ value: 'company_tutor', label: 'Tuteurs entreprise' },
]

const KIND_NOUN: Record<KindFilter, CountNoun> = {
	all: PARTICIPANT_NOUN,
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

export function ParticipantsListUI({
	rows,
	total,
	search,
	isLoading,
	error,
	kind,
	status,
	canManage,
	page,
	pageCount,
	onSearchChange,
	onResetFilters,
	onKindChange,
	onStatusChange,
	onPageChange,
	onCreate,
	onOpen,
}: ParticipantsListUIProps) {
	const filtered =
		search.trim() !== '' || kind !== undefined || status !== undefined

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title="Participants"
				description={
					total === undefined
						? undefined
						: countSummary(total, KIND_NOUN[kind ?? 'all'], status)
				}
				actions={
					canManage ? (
						<Button type="button" size="sm" onClick={onCreate}>
							<Plus aria-hidden /> Nouveau participant
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
				select={{
					value: kind ?? 'all',
					label: 'Filtrer par type',
					options: KIND_OPTIONS,
					onChange: (next) => onKindChange(next === 'all' ? undefined : next),
				}}
			/>
			<FormError message={error} />
			{isLoading || error ? null : rows.length === 0 ? (
				<ListEmpty
					title="Aucun participant ne correspond."
					hint={
						filtered
							? 'Vérifie l’orthographe, le statut ou le type sélectionné.'
							: undefined
					}
					onReset={filtered ? onResetFilters : undefined}
				/>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Nom</TableHead>
								<TableHead>E-mail</TableHead>
								<TableHead>Type</TableHead>
								<TableHead>Entreprise</TableHead>
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
									<TableCell className="text-muted-foreground">
										{row.email}
									</TableCell>
									<TableCell>
										{row.kinds.length === 0
											? '—'
											: row.kinds.map((value) => KIND_LABELS[value]).join(', ')}
									</TableCell>
									<TableCell>
										{row.hostCompanyNames.length === 0
											? '—'
											: row.hostCompanyNames.join(', ')}
									</TableCell>
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
