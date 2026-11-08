import { Ban, Check, MoreHorizontal, Search, X } from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
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
import { findMember } from '#/pages/hr/lib/rh/absences'
import { formatShort, todayIso } from '#/pages/hr/lib/rh/dates'
import {
	describeSpan,
	equivalentMinutes,
	extraMinutes,
	findSurcharge,
	formatDuration,
} from '#/pages/hr/lib/rh/extra'
import {
	canCancel,
	dayCaption,
	filterHistory,
	HISTORY_ALL,
	type HistoryFilters,
} from '#/pages/hr/lib/rh/extra-view'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { ExtraPeriod, RhState } from '#/pages/hr/lib/rh/types'
import { RateBadge } from '#/pages/hr/ui/rh/extra/rate-badge'
import {
	AbsenceStatusBadge,
	STATUS_LABELS,
} from '#/pages/hr/ui/rh/shared/absence-status-badge'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'

const DEFAULT_FILTERS: HistoryFilters = {
	query: '',
	status: HISTORY_ALL,
	memberId: HISTORY_ALL,
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
	onApprove: (extra: ExtraPeriod) => void
	onReject: (extra: ExtraPeriod) => void
	onCancel: (extra: ExtraPeriod) => void
	onCreate: () => void
}) {
	const [query, setQuery] = useState('')
	const filters: HistoryFilters = { query, status, memberId }
	function setFilters(next: HistoryFilters) {
		setQuery(next.query)
		if (next.status !== status || next.memberId !== memberId)
			onFiltersChange({ status: next.status, memberId: next.memberId })
	}
	const rows = filterHistory(state, filters)
	const filtered =
		query.trim() !== '' || status !== HISTORY_ALL || memberId !== HISTORY_ALL
	const memberIds = new Set(state.extras.map((extra) => extra.memberId))
	memberIds.add(memberId)
	const members = state.members
		.filter((member) => memberIds.has(member.id))
		.sort((left, right) => fullName(left).localeCompare(fullName(right)))
	const today = todayIso()

	return (
		<section aria-labelledby="extra-history-title">
			<div className="mb-3 flex items-baseline gap-2">
				<h2 id="extra-history-title" className="font-semibold text-base">
					Historique
				</h2>
				<span className="text-muted-foreground text-sm tabular">
					{rows.length} déclaration{rows.length > 1 ? 's' : ''}
				</span>
			</div>
			<div className="mb-3 flex flex-wrap items-center gap-2">
				<div className="relative w-full max-w-60">
					<Search
						className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
						aria-hidden
					/>
					<Input
						type="search"
						value={query}
						onChange={(event) =>
							setFilters({ ...filters, query: event.target.value })
						}
						placeholder="Rechercher un membre"
						aria-label="Rechercher un membre"
						className="h-8 bg-card pl-8"
					/>
				</div>
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={status}
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
				<Select
					value={memberId}
					onValueChange={(next) => setFilters({ ...filters, memberId: next })}
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
			</div>
			{rows.length === 0 ? (
				<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<p className="font-medium">
						{filtered
							? 'Aucune déclaration ne correspond à ces filtres.'
							: 'Aucune heure extra déclarée pour le moment.'}
					</p>
					<p className="mt-1 text-[13px] text-muted-foreground">
						{filtered
							? 'Élargis la recherche ou retire un filtre.'
							: 'Les heures faites en plus de l’horaire habituel apparaîtront ici.'}
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
							<RequirePermission permission="MANAGE_PLANNING">
								<Button type="button" size="sm" onClick={onCreate}>
									Déclarer des heures
								</Button>
							</RequirePermission>
						)}
					</div>
				</div>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Membre</TableHead>
								<TableHead>Date</TableHead>
								<TableHead>Plage</TableHead>
								<TableHead>Durée</TableHead>
								<TableHead>Majoration</TableHead>
								<TableHead>Équivalent payé</TableHead>
								<TableHead>Statut</TableHead>
								<TableHead>Décision</TableHead>
								<TableHead className="w-12 pr-4">
									<span className="sr-only">Actions</span>
								</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{rows.map((extra) => (
								<HistoryRow
									key={extra.id}
									state={state}
									extra={extra}
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
	extra,
	today,
	onApprove,
	onReject,
	onCancel,
}: {
	state: RhState
	extra: ExtraPeriod
	today: string
	onApprove: (extra: ExtraPeriod) => void
	onReject: (extra: ExtraPeriod) => void
	onCancel: (extra: ExtraPeriod) => void
}) {
	const member = findMember(state, extra.memberId)
	if (!member) return null
	const name = fullName(member)
	const decider = findMember(state, extra.decidedById)
	const surcharge = findSurcharge(state, extra.surchargeId)
	const pending = extra.status === 'pending'
	const cancellable = canCancel(extra, today)
	return (
		<TableRow>
			<TableCell className="py-2.5 pl-4">
				<div className="flex items-center gap-2.5">
					<MemberAvatar member={member} size="md" />
					<span className="font-medium">{name}</span>
				</div>
			</TableCell>
			<TableCell>
				<span className="block">{formatShort(extra.date)}</span>
				<span className="block text-muted-foreground text-xs">
					{dayCaption(extra.date)}
				</span>
			</TableCell>
			<TableCell className="tabular">{describeSpan(extra)}</TableCell>
			<TableCell className="tabular">
				{formatDuration(extraMinutes(extra))}
			</TableCell>
			<TableCell>
				<RateBadge rate={extra.rate} />
				{extra.rate > 0 && (
					<span className="mt-0.5 block max-w-40 truncate text-muted-foreground text-xs">
						{surcharge?.name ?? 'Taux libre'}
					</span>
				)}
			</TableCell>
			<TableCell
				className={extra.rate > 0 ? 'tabular' : 'tabular text-muted-foreground'}
			>
				{formatDuration(equivalentMinutes(extra))}
			</TableCell>
			<TableCell>
				<AbsenceStatusBadge status={extra.status} />
			</TableCell>
			<TableCell className="max-w-64 whitespace-normal">
				{pending || extra.decidedAt === null ? (
					<span className="text-muted-foreground">—</span>
				) : (
					<>
						<span className="block">
							{decider ? decider.firstName : 'Automatique'}
							<span className="text-muted-foreground">
								{' '}
								· {formatShort(todayIso(new Date(extra.decidedAt)))}
							</span>
						</span>
						{extra.decisionComment !== '' && (
							<span
								title={extra.decisionComment}
								className="line-clamp-2 text-muted-foreground text-xs"
							>
								{extra.decisionComment}
							</span>
						)}
					</>
				)}
			</TableCell>
			<TableCell className="pr-4 text-right">
				{(pending || cancellable) && (
					<RequirePermission permission="MANAGE_PLANNING">
						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									aria-label={`Actions pour la déclaration de ${name}, ${formatShort(extra.date)}`}
								>
									<MoreHorizontal />
								</Button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="end">
								{pending && (
									<>
										<DropdownMenuItem onSelect={() => onApprove(extra)}>
											<Check />
											Valider
										</DropdownMenuItem>
										<DropdownMenuItem onSelect={() => onReject(extra)}>
											<X />
											Refuser
										</DropdownMenuItem>
									</>
								)}
								{pending && cancellable && <DropdownMenuSeparator />}
								{cancellable && (
									<DropdownMenuItem
										variant="destructive"
										onSelect={() => onCancel(extra)}
									>
										<Ban />
										Annuler la déclaration
									</DropdownMenuItem>
								)}
							</DropdownMenuContent>
						</DropdownMenu>
					</RequirePermission>
				)}
			</TableCell>
		</TableRow>
	)
}
