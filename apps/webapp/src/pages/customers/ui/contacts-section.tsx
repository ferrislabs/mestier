import { Mail, Pencil, Phone, Plus, Star, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import type { CustomerContact } from '#/hooks/use-customers'
import type { ContactForm } from '#/pages/customers/lib/account-form'
import { contactsSummary, TAB_INFO } from '#/pages/customers/lib/rubriques'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { ContactFormDialog } from '#/pages/customers/ui/contact-form-dialog'
import { Dash } from '#/pages/customers/ui/detail-card'
import { RecordCard, RecordField } from '#/pages/customers/ui/record-card'

const LINK =
	'inline-flex max-w-full items-center gap-1.5 rounded-sm outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring'

export function contactFormOf(contact?: CustomerContact): ContactForm {
	return {
		firstName: contact?.first_name ?? '',
		lastName: contact?.last_name ?? '',
		role: contact?.role ?? '',
		email: contact?.email ?? '',
		phone: contact?.phone ?? '',
	}
}

function nameOf(contact: CustomerContact): string {
	return (
		`${contact.first_name} ${contact.last_name}`.trim() || 'Contact sans nom'
	)
}

interface ContactActions {
	isSaving: boolean
	isDeleting: (contactId: string) => boolean
	error: string | null
	onSave: (contact: CustomerContact | null, form: ContactForm) => Promise<void>
	onMakePrimary: (contact: CustomerContact) => void
	onDelete: (contact: CustomerContact) => void
}

function ContactCard({
	contact,
	actions,
}: {
	contact: CustomerContact
	actions: ContactActions
}) {
	const [editing, setEditing] = useState(false)
	const [deleting, setDeleting] = useState(false)
	const name = nameOf(contact)
	return (
		<RecordCard
			title={name}
			subtitle={contact.role || undefined}
			badges={
				contact.is_primary && (
					<StatusBadge tone="info">
						<Star className="size-3" aria-hidden />
						Contact principal
					</StatusBadge>
				)
			}
			actions={
				<RequirePermission permission="MANAGE_CUSTOMERS">
					<ContactFormDialog
						open={editing}
						onOpenChange={setEditing}
						title={`Modifier ${name}`}
						initial={contactFormOf(contact)}
						isEditing
						isPending={actions.isSaving}
						error={actions.error}
						onSubmit={async (form) => {
							await actions.onSave(contact, form)
							setEditing(false)
						}}
						trigger={
							<Button variant="ghost" size="sm">
								<Pencil aria-hidden /> Modifier
								<span className="sr-only"> {name}</span>
							</Button>
						}
					/>
					{!contact.is_primary && (
						<Button
							variant="ghost"
							size="sm"
							onClick={() => actions.onMakePrimary(contact)}
						>
							<Star aria-hidden /> Définir comme contact principal
							<span className="sr-only"> : {name}</span>
						</Button>
					)}
					<Button
						variant="ghost"
						size="sm"
						className="text-state-alert hover:text-state-alert"
						disabled={actions.isDeleting(contact.id)}
						onClick={() => setDeleting(true)}
					>
						<Trash2 aria-hidden /> Supprimer
						<span className="sr-only"> {name}</span>
					</Button>
					<ConfirmDialog
						open={deleting}
						onOpenChange={setDeleting}
						title={`Supprimer ${name} ?`}
						description="Les devis et factures déjà émis conservent le contact figé à l’émission."
						confirmLabel="Supprimer"
						destructive
						onConfirm={() => {
							actions.onDelete(contact)
							setDeleting(false)
						}}
					/>
				</RequirePermission>
			}
		>
			<dl className="grid gap-x-4 gap-y-2 sm:grid-cols-2">
				<RecordField label="E-mail">
					{contact.email ? (
						<a
							href={`mailto:${contact.email}`}
							className={`${LINK} text-primary`}
						>
							<Mail className="size-3.5 shrink-0" aria-hidden />
							<span className="truncate">{contact.email}</span>
						</a>
					) : (
						<Dash />
					)}
				</RecordField>
				<RecordField label="Téléphone">
					{contact.phone ? (
						<a
							href={`tel:${contact.phone.replace(/[\s.-]/g, '')}`}
							className={`${LINK} tabular`}
						>
							<Phone
								className="size-3.5 shrink-0 text-muted-foreground"
								aria-hidden
							/>
							{contact.phone}
						</a>
					) : (
						<Dash />
					)}
				</RecordField>
			</dl>
		</RecordCard>
	)
}

export function ContactsSection({
	contacts,
	isLoading,
	loadError,
	onRetry,
	actions,
}: {
	contacts: readonly CustomerContact[]
	isLoading: boolean
	loadError: string | null
	onRetry: () => void
	actions: ContactActions
}) {
	const [adding, setAdding] = useState(false)
	const count = contacts.length
	return (
		<RubriqueLayout
			title={TAB_INFO.contacts.label}
			description={`${contactsSummary(count)}. Le contact principal est repris sur les devis et les factures.`}
			actions={
				<RequirePermission permission="MANAGE_CUSTOMERS">
					<ContactFormDialog
						open={adding}
						onOpenChange={setAdding}
						title="Ajouter un contact"
						initial={contactFormOf()}
						isEditing={false}
						isPending={actions.isSaving}
						error={actions.error}
						onSubmit={async (form) => {
							await actions.onSave(null, form)
							setAdding(false)
						}}
						trigger={
							<Button size="sm">
								<Plus aria-hidden /> Ajouter un contact
							</Button>
						}
					/>
				</RequirePermission>
			}
		>
			{loadError ? (
				<div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-[13px] text-state-alert">
					<p className="font-medium">{loadError}</p>
					<Button type="button" variant="outline" size="sm" onClick={onRetry}>
						Réessayer
					</Button>
				</div>
			) : null}
			{isLoading ? (
				<EmptyState title="Chargement des contacts…" />
			) : count === 0 ? (
				<EmptyState
					title="Aucun contact"
					hint="Ajoute un interlocuteur : il sera repris sur les devis et factures."
				/>
			) : (
				<ul className="grid gap-3">
					{contacts.map((contact) => (
						<li key={contact.id}>
							<ContactCard contact={contact} actions={actions} />
						</li>
					))}
				</ul>
			)}
		</RubriqueLayout>
	)
}
