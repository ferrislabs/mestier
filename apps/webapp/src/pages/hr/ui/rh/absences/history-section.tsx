import { Ban, Check, MoreHorizontal, Search, X } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectSeparator,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { useHasPermission } from '#/hooks/use-permissions'
import { absenceDays, findMember, findType } from '#/pages/hr/lib/rh/absences'
import {
	canCancel,
	canCancelOwn,
	filterHistory,
	formatDayCount,
	HISTORY_ALL,
	type HistoryFilters,
	partsNote,
} from '#/pages/hr/lib/rh/absences-view'
import { formatRange, formatShort, todayIso } from '#/pages/hr/lib/rh/dates'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { Absence, RhState } from '#/pages/hr/lib/rh/types'
import {
	AbsenceStatusBadge,
	STATUS_LABELS,
} from '#/pages/hr/ui/rh/shared/absence-status-badge'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import {
	CanRequest,
	useOwnRequestsOnly,
} from '#/pages/hr/ui/rh/shared/request-access'
import { useRhActorId } from '#/pages/hr/ui/rh/shared/rh-context'
import { TypeChip, TypeDot } from '#/pages/hr/ui/rh/shared/type-chip'

const DEFAULT_FILTERS: HistoryFilters = {
	query: '',
	status: HISTORY_ALL,
	memberId: HISTORY_ALL,
	typeId: HISTORY_ALL,
}

const STATUS_FILTERS = [
	{ value: HISTORY_ALL, label: 'Toutes' },
	{ value: 'pending', label: STATUS_LABELS.pending },
	{ value: 'approved', label: 'Validées' },
	{ value: 'rejected', label: 'Refusées' },
	{ value: 'cancelled', label: 'Annulées' },
] as const

export function HistorySection({
	state,
	status,
	memberId,
	onFiltersChange,
	onApprove,
	onReject,
	onCancel,
	onCreate,
}: {
	state: RhState
	status: HistoryFilters['status']
	memberId: string
	onFiltersChange: (patch: {
		status: HistoryFilters['status']
		memberId: string
	}) => void
	onApprove: (absence: Absence) => void
	onReject: (absence: Absence) => void
	onCancel: (absence: Absence) => void
	onCreate: () => void
}) {
	const mine = useOwnRequestsOnly('absence')
	const [local, setLocal] = useState({ query: '', typeId: HISTORY_ALL })
	const filters: HistoryFilters = { ...local, status, memberId }
	function setFilters(next: HistoryFilters) {
		setLocal({ query: next.query, typeId: next.typeId })
		if (next.status !== status || next.memberId !== memberId)
			onFiltersChange({ status: next.status, memberId: next.memberId })
	}
	const rows = filterHistory(state, filters)
	const filtered =
		filters.query.trim() !== '' ||
		filters.status !== HISTORY_ALL ||
		filters.memberId !== HISTORY_ALL ||
		filters.typeId !== HISTORY_ALL
	const memberIds = new Set(state.absences.map((absence) => absence.memberId))
	const members = state.members
		.filter((member) => memberIds.has(member.id))
		.sort((left, right) => fullName(left).localeCompare(fullName(right)))
	const today = todayIso()

	return (
		<section aria-labelledby="history-title">
			<div className="mb-3 flex items-baseline gap-2">
				<h2 id="history-title" className="font-semibold text-base">
					Historique
				</h2>
				<span className="text-muted-foreground text-sm tabular">
					{rows.length} demande{rows.length > 1 ? 's' : ''}
				</span>
			</div>
			<div className="mb-3 flex flex-wrap items-center gap-2">
				{!mine && (
					<div className="relative w-full max-w-60">
						<Search
							className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
							aria-hidden
						/>
						<Input
							type="search"
							value={filters.query}
							onChange={(event) =>
								setFilters({ ...filters, query: event.target.value })
							}
							placeholder="Rechercher un membre"
							aria-label="Rechercher un membre"
							className="h-8 bg-card pl-8"
						/>
					</div>
				)}
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={filters.status}
					onValueChange={(value) =>
						value &&
						setFilters({
							...filters,
							status: value as HistoryFilters['status'],
						})
					}
					aria-label="Filtrer par statut"
				>
					{STATUS_FILTERS.map((entry) => (
						<ToggleGroupItem key={entry.value} value={entry.value}>
							{entry.label}
						</ToggleGroupItem>
					))}
				</ToggleGroup>
				{!mine && (
					<Select
						value={filters.memberId}
						onValueChange={(memberId) => setFilters({ ...filters, memberId })}
					>
						<SelectTrigger
							size="sm"
							aria-label="Filtrer par membre"
							className="min-w-44 bg-card"
						>
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value={HISTORY_ALL}>Tous les membres</SelectItem>
							<SelectSeparator />
							{members.map((member) => (
								<SelectItem key={member.id} value={member.id}>
									<MemberAvatar member={member} size="sm" />
									{fullName(member)}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				)}
				<Select
					value={filters.typeId}
					onValueChange={(typeId) => setFilters({ ...filters, typeId })}
				>
					<SelectTrigger
						size="sm"
						aria-label="Filtrer par motif"
						className="min-w-44 bg-card"
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={HISTORY_ALL}>Tous les motifs</SelectItem>
						<SelectSeparator />
						{state.absenceTypes.map((type) => (
							<SelectItem key={type.id} value={type.id}>
								<TypeDot hue={type.hue} />
								{type.name}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
			{rows.length === 0 ? (
				<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<p className="font-medium">
						{filtered
							? 'Aucune demande ne correspond à ces filtres.'
							: 'Aucune demande d’absence pour le moment.'}
					</p>
					<p className="mt-1 text-[13px] text-muted-foreground">
						{filtered
							? 'Élargis la recherche ou retire un filtre.'
							: mine
								? 'Tes demandes apparaîtront ici dès que tu en feras une.'
								: 'Les absences posées par l’équipe apparaîtront ici.'}
					</p>
					<div className="mt-4 flex justify-center">
						{filtered ? (
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={() => setFilters(DEFAULT_FILTERS)}
							>
								Réinitialiser les filtres
							</Button>
						) : (
							<CanRequest kind="absence">
								<Button type="button" size="sm" onClick={onCreate}>
									Poser une absence
								</Button>
							</CanRequest>
						)}
					</div>
				</div>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Membre</TableHead>
								<TableHead>Motif</TableHead>
								<TableHead>Période</TableHead>
								<TableHead>Durée</TableHead>
								<TableHead>Statut</TableHead>
								<TableHead>Décision</TableHead>
								<TableHead className="w-12 pr-4">
									<span className="sr-only">Actions</span>
								</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{rows.map((absence) => (
								<HistoryRow
									key={absence.id}
									state={state}
									absence={absence}
									today={today}
									onApprove={onApprove}
									onReject={onReject}
									onCancel={onCancel}
								/>
							))}
						</TableBody>
					</Table>
				</div>
			)}
		</section>
	)
}

function HistoryRow({
	state,
	absence,
	today,
	onApprove,
	onReject,
	onCancel,
}: {
	state: RhState
	absence: Absence
	today: string
	onApprove: (absence: Absence) => void
	onReject: (absence: Absence) => void
	onCancel: (absence: Absence) => void
}) {
	const canDecide = useHasPermission('MANAGE_PLANNING')
	const actorId = useRhActorId()
	const member = findMember(state, absence.memberId)
	const type = findType(state, absence.typeId)
	if (!member || !type) return null
	const name = fullName(member)
	const note = partsNote(absence)
	const decider = findMember(state, absence.decidedById)
	const pending = absence.status === 'pending'
	const decidable = pending && canDecide
	const cancellable =
		(canDecide && canCancel(absence, today)) ||
		canCancelOwn(absence, actorId, today)
	return (
		<TableRow>
			<TableCell className="py-2.5 pl-4">
				<div className="flex items-center gap-2.5">
					<MemberAvatar member={member} size="md" />
					<span className="font-medium">{name}</span>
				</div>
			</TableCell>
			<TableCell>
				<TypeChip type={type} />
			</TableCell>
			<TableCell>
				<span className="block">
					{formatRange(absence.startDate, absence.endDate)}
				</span>
				{note && (
					<span className="block text-muted-foreground text-xs">{note}</span>
				)}
			</TableCell>
			<TableCell className="tabular">
				{formatDayCount(absenceDays(absence, member, type))}
			</TableCell>
			<TableCell>
				<AbsenceStatusBadge status={absence.status} />
			</TableCell>
			<TableCell className="max-w-72 whitespace-normal">
				{pending || absence.decidedAt === null ? (
					<span className="text-muted-foreground">—</span>
				) : (
					<>
						<span className="block">
							{decider ? decider.firstName : 'Automatique'}
							<span className="text-muted-foreground">
								{' '}
								· {formatShort(todayIso(new Date(absence.decidedAt)))}
							</span>
						</span>
						{absence.decisionComment !== '' && (
							<span
								title={absence.decisionComment}
								className="line-clamp-2 text-muted-foreground text-xs"
							>
								{absence.decisionComment}
							</span>
						)}
					</>
				)}
			</TableCell>
			<TableCell className="pr-4 text-right">
				{(decidable || cancellable) && (
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								aria-label={`Actions pour la demande de ${name}, ${formatRange(absence.startDate, absence.endDate)}`}
							>
								<MoreHorizontal />
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							{decidable && (
								<>
									<DropdownMenuItem onSelect={() => onApprove(absence)}>
										<Check />
										Valider
									</DropdownMenuItem>
									<DropdownMenuItem onSelect={() => onReject(absence)}>
										<X />
										Refuser
									</DropdownMenuItem>
								</>
							)}
							{decidable && cancellable && <DropdownMenuSeparator />}
							{cancellable && (
								<DropdownMenuItem
									variant="destructive"
									onSelect={() => onCancel(absence)}
								>
									<Ban />
									Annuler la demande
								</DropdownMenuItem>
							)}
						</DropdownMenuContent>
					</DropdownMenu>
				)}
			</TableCell>
		</TableRow>
	)
}
