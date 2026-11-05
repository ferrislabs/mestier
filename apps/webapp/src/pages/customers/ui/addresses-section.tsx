import { Image, Pencil, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import type { CustomerContext } from '#/hooks/use-customers'
import { EMPTY_ADDRESS } from '#/pages/customers/lib/account-form'
import { addressesSummary, TAB_INFO } from '#/pages/customers/lib/rubriques'
import {
	type AddressDraft,
	AddressFormDialog,
} from '#/pages/customers/ui/address-form-dialog'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { RecordCard } from '#/pages/customers/ui/record-card'

export function addressDraftOf(context?: CustomerContext): AddressDraft {
	return {
		label: context?.label ?? '',
		address: {
			...EMPTY_ADDRESS,
			street: context?.address_line ?? '',
			postalCode: context?.postal_code ?? '',
			city: context?.city ?? '',
		},
		photoKey: context?.photo_key ?? '',
	}
}

export function formatContextAddress(context: CustomerContext): string {
	const town = [context.postal_code, context.city].filter(Boolean).join(' ')
	return [context.address_line, town].filter(Boolean).join(', ')
}

interface AddressActions {
	isSaving: boolean
	isUploadingPhoto: boolean
	isDeleting: (contextId: string) => boolean
	error: string | null
	onUploadPhoto: (file: File) => Promise<string>
	onSave: (
		context: CustomerContext | null,
		draft: AddressDraft,
	) => Promise<void>
	onDelete: (context: CustomerContext) => void
}

function AddressCard({
	context,
	actions,
}: {
	context: CustomerContext
	actions: AddressActions
}) {
	const [editing, setEditing] = useState(false)
	const [deleting, setDeleting] = useState(false)
	const formatted = formatContextAddress(context)
	return (
		<RecordCard
			title={context.label}
			badges={
				context.photo_key && (
					<StatusBadge tone="neutral">
						<Image className="size-3" aria-hidden />
						Photo
					</StatusBadge>
				)
			}
			actions={
				<RequirePermission permission="MANAGE_CUSTOMERS">
					<AddressFormDialog
						open={editing}
						onOpenChange={setEditing}
						title={`Modifier ${context.label}`}
						initial={addressDraftOf(context)}
						isEditing
						isPending={actions.isSaving}
						isUploadingPhoto={actions.isUploadingPhoto}
						error={actions.error}
						onUploadPhoto={actions.onUploadPhoto}
						onSubmit={async (draft) => {
							await actions.onSave(context, draft)
							setEditing(false)
						}}
						trigger={
							<Button variant="ghost" size="sm">
								<Pencil aria-hidden /> Modifier
								<span className="sr-only"> {context.label}</span>
							</Button>
						}
					/>
					<Button
						variant="ghost"
						size="sm"
						className="text-state-alert hover:text-state-alert"
						disabled={actions.isDeleting(context.id)}
						onClick={() => setDeleting(true)}
					>
						<Trash2 aria-hidden /> Supprimer
						<span className="sr-only"> {context.label}</span>
					</Button>
					<ConfirmDialog
						open={deleting}
						onOpenChange={setDeleting}
						title={`Supprimer ${context.label} ?`}
						description="Les devis et factures déjà émis conservent l’adresse figée à l’émission."
						confirmLabel="Supprimer"
						destructive
						onConfirm={() => {
							actions.onDelete(context)
							setDeleting(false)
						}}
					/>
				</RequirePermission>
			}
		>
			<address className="grid gap-0.5 text-[13px] not-italic">
				{formatted ? (
					<span>{formatted}</span>
				) : (
					<span className="text-muted-foreground">
						Aucune adresse renseignée
					</span>
				)}
			</address>
		</RecordCard>
	)
}

export function AddressesSection({
	contexts,
	isLoading,
	loadError,
	onRetry,
	actions,
}: {
	contexts: readonly CustomerContext[]
	isLoading: boolean
	loadError: string | null
	onRetry: () => void
	actions: AddressActions
}) {
	const [adding, setAdding] = useState(false)
	const count = contexts.length
	return (
		<RubriqueLayout
			title={TAB_INFO.adresses.label}
			description={`${addressesSummary(count)}. Siège, chantiers et autres lieux associés au client.`}
			actions={
				<RequirePermission permission="MANAGE_CUSTOMERS">
					<AddressFormDialog
						open={adding}
						onOpenChange={setAdding}
						title="Ajouter une adresse"
						initial={addressDraftOf()}
						isEditing={false}
						isPending={actions.isSaving}
						isUploadingPhoto={actions.isUploadingPhoto}
						error={actions.error}
						onUploadPhoto={actions.onUploadPhoto}
						onSubmit={async (draft) => {
							await actions.onSave(null, draft)
							setAdding(false)
						}}
						trigger={
							<Button size="sm">
								<Plus aria-hidden /> Ajouter une adresse
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
				<EmptyState title="Chargement des adresses…" />
			) : count === 0 ? (
				<EmptyState
					title="Aucune adresse"
					hint="Ajoute l’adresse du client pour qu’elle figure sur les devis et factures."
				/>
			) : (
				<ul className="grid gap-3">
					{contexts.map((context) => (
						<li key={context.id}>
							<AddressCard context={context} actions={actions} />
						</li>
					))}
				</ul>
			)}
		</RubriqueLayout>
	)
}
