import { Check, Copy, KeyRound, Mail } from 'lucide-react'
import { type ReactNode, useEffect, useState } from 'react'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { cn } from '#/lib/utils'
import { formatDateTime, formatRelative } from '#/pages/support/lib/format'
import { SUPPORT_PRIORITIES } from '#/pages/support/lib/priorities'
import type {
	SupportPriority,
	SupportPropertiesUIProps,
	SupportStatus,
	SupportTicketVM,
} from '#/pages/support/types'
import { AssigneeList, AssigneeMenu } from '#/pages/support/ui/assignee-picker'
import { BadgeEditor } from '#/pages/support/ui/badge-editor'
import { PersonAvatar } from '#/pages/support/ui/member-avatar'
import { PriorityIcon } from '#/pages/support/ui/priority-badge'
import { StatusDot } from '#/pages/support/ui/status-badge'

const OTHER_TICKET =
	'-mx-2 flex w-[calc(100%+1rem)] items-center gap-2 rounded-md px-2 py-1 text-left text-sm outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring'

const SELECT_TRIGGER =
	'-mx-2 h-8 w-[calc(100%+1rem)] border-transparent bg-transparent px-2 shadow-none hover:bg-accent dark:bg-transparent dark:hover:bg-accent'

function OtherTicketLabel({ ticket }: { ticket: SupportTicketVM }) {
	return (
		<>
			<StatusDot status={ticket.status} />
			<span className="shrink-0 font-mono text-muted-foreground text-xs">
				{ticket.numberLabel}
			</span>
			<span className="truncate">{ticket.subject}</span>
		</>
	)
}

function Group({
	title,
	hint,
	children,
}: {
	title: string
	hint?: string
	children: ReactNode
}) {
	return (
		<section aria-label={title} className="grid gap-2 px-4 py-4">
			<h3 className="font-medium text-muted-foreground text-xs">{title}</h3>
			{children}
			{hint && <p className="text-muted-foreground text-xs">{hint}</p>}
		</section>
	)
}

function Field({
	label,
	action,
	children,
}: {
	label: string
	action?: ReactNode
	children: ReactNode
}) {
	return (
		<div className="grid gap-1">
			<div className="flex items-center justify-between gap-2">
				<p className="font-medium text-muted-foreground text-xs">{label}</p>
				{action}
			</div>
			{children}
		</div>
	)
}

function Row({ label, children }: { label: string; children: ReactNode }) {
	return (
		<div className="grid min-h-8 grid-cols-[5.5rem_minmax(0,1fr)] items-center gap-2 text-sm">
			<dt className="text-muted-foreground">{label}</dt>
			<dd className="min-w-0">{children}</dd>
		</div>
	)
}

function CopyButton({ value, label }: { value: string; label: string }) {
	const [copied, setCopied] = useState(false)
	useEffect(() => {
		if (!copied) return
		const timer = window.setTimeout(() => setCopied(false), 1500)
		return () => window.clearTimeout(timer)
	}, [copied])
	return (
		<button
			type="button"
			aria-label={label}
			onClick={() => {
				navigator.clipboard?.writeText(value).catch(() => undefined)
				setCopied(true)
			}}
			className="rounded-sm p-0.5 text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
		>
			{copied ? (
				<Check className="size-3.5" aria-hidden />
			) : (
				<Copy className="size-3.5" aria-hidden />
			)}
		</button>
	)
}

export function TicketProperties({
	ticket,
	variant,
	canManage,
	statusOptions,
	members,
	knownBadges,
	otherTickets,
	error,
	onOpenOtherTicket,
	onStatusChange,
	onPriorityChange,
	onAssigneeChange,
	onAddBadge,
	onRemoveBadge,
}: SupportPropertiesUIProps) {
	return (
		<aside
			aria-label="Propriétés du ticket"
			className={cn(
				'divide-y divide-border',
				variant === 'card' &&
					'rounded-lg border border-border bg-card lg:sticky lg:top-6',
			)}
		>
			<section aria-label="Propriétés" className="grid gap-4 px-4 py-4">
				<Field label="Statut">
					<Select
						value={ticket.status}
						disabled={!canManage}
						onValueChange={(value) => onStatusChange(value as SupportStatus)}
					>
						<SelectTrigger
							aria-label="Statut du ticket"
							className={SELECT_TRIGGER}
						>
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{statusOptions.map((status) => (
								<SelectItem key={status.status} value={status.status}>
									<StatusDot status={status.status} />
									{status.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</Field>
				<Field label="Priorité">
					<Select
						value={ticket.priority}
						disabled={!canManage}
						onValueChange={(value) =>
							onPriorityChange(value as SupportPriority)
						}
					>
						<SelectTrigger
							aria-label="Priorité du ticket"
							className={SELECT_TRIGGER}
						>
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{SUPPORT_PRIORITIES.map((priority) => (
								<SelectItem key={priority.id} value={priority.id}>
									<PriorityIcon priority={priority.id} />
									{priority.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</Field>
				<Field
					label="Assignés"
					action={
						<AssigneeMenu
							assigneeId={ticket.assignee?.id ?? null}
							members={members}
							disabled={!canManage}
							onChange={onAssigneeChange}
						/>
					}
				>
					<AssigneeList assignee={ticket.assignee} />
				</Field>
				<Field label="Badges">
					<BadgeEditor
						badges={ticket.badges}
						suggestions={knownBadges}
						disabled={!canManage}
						onAdd={onAddBadge}
						onRemove={onRemoveBadge}
					/>
				</Field>
				{error ? <p className="text-destructive text-xs">{error}</p> : null}
			</section>
			<Group title="Client">
				<div className="flex items-center gap-2.5">
					<PersonAvatar name={ticket.requester.name} hue={null} size="lg" />
					<div className="min-w-0 flex-1">
						<p className="truncate font-medium text-sm">
							{ticket.requester.name}
						</p>
						{ticket.requester.email ? (
							<a
								href={`mailto:${ticket.requester.email}`}
								className="flex min-w-0 items-center gap-1 text-muted-foreground text-xs hover:text-foreground"
							>
								<Mail className="size-3 shrink-0" aria-hidden />
								<span className="truncate">{ticket.requester.email}</span>
							</a>
						) : null}
					</div>
				</div>
				{otherTickets.length > 0 && (
					<div className="grid gap-1 pt-1">
						<p className="text-muted-foreground text-xs">
							{otherTickets.length === 1
								? 'Autre ticket'
								: `${otherTickets.length} autres tickets`}
						</p>
						<ul className="grid gap-0.5">
							{otherTickets.slice(0, 4).map((other) => (
								<li key={other.id}>
									<button
										type="button"
										onClick={() => onOpenOtherTicket(other.id)}
										className={OTHER_TICKET}
									>
										<OtherTicketLabel ticket={other} />
									</button>
								</li>
							))}
						</ul>
					</div>
				)}
			</Group>
			<Group
				title="Champs personnalisés"
				hint="Transmis par la plateforme à la création du ticket."
			>
				{ticket.customFields.length === 0 ? (
					<p className="text-muted-foreground text-sm">Aucun champ transmis.</p>
				) : (
					<dl className="grid gap-0.5">
						{ticket.customFields.map((field) => (
							<Row key={field.key} label={field.key}>
								<span title={field.key} className="block truncate">
									{field.value}
								</span>
							</Row>
						))}
					</dl>
				)}
			</Group>
			<Group title="Origine">
				<dl className="grid gap-0.5">
					<Row label="Source">
						<span className="flex min-w-0 items-center gap-1.5">
							<KeyRound
								className="size-3.5 shrink-0 text-muted-foreground"
								aria-hidden
							/>
							<span className="truncate">{ticket.sourceLabel}</span>
						</span>
					</Row>
					{ticket.externalId && (
						<Row label="ID externe">
							<span className="flex min-w-0 items-center gap-1">
								<span className="truncate font-mono text-xs">
									{ticket.externalId}
								</span>
								<CopyButton
									value={ticket.externalId}
									label="Copier l’ID externe"
								/>
							</span>
						</Row>
					)}
					<Row label="Créé">
						<span title={formatDateTime(ticket.createdAt)}>
							{formatRelative(ticket.createdAt)}
						</span>
					</Row>
					<Row label="Mis à jour">
						<span title={formatDateTime(ticket.updatedAt)}>
							{formatRelative(ticket.updatedAt)}
						</span>
					</Row>
				</dl>
			</Group>
		</aside>
	)
}
