import {
	Archive,
	ArchiveRestore,
	ExternalLink,
	Pencil,
	UserCheck,
} from 'lucide-react'
import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import type { Customer } from '#/hooks/use-customers'
import type { AccountExtras } from '#/pages/customers/feature/mocks/account-extras'
import { findPerson } from '#/pages/customers/feature/mocks/people'
import type { AccountForm } from '#/pages/customers/lib/account-form'
import { openDealsSentence } from '#/pages/customers/lib/accounts'
import { formatDate } from '#/pages/customers/lib/format'
import { formatSiret, TAB_INFO } from '#/pages/customers/lib/rubriques'
import {
	ACCOUNT_KIND_LABELS,
	accountStatusOf,
	kindOf,
	LEAD_SOURCE_LABELS,
} from '#/pages/customers/types'
import { PersonAvatar } from '#/pages/customers/ui/avatars'
import { CustomerEditDialog } from '#/pages/customers/ui/customer-edit-dialog'
import {
	Dash,
	DetailCard,
	DetailList,
	DetailRow,
} from '#/pages/customers/ui/detail-card'

function websiteHref(website: string): string {
	return /^https?:\/\//i.test(website) ? website : `https://${website}`
}

interface IdentitySectionProps {
	customer: Customer
	extras: AccountExtras
	defaultTerms: number
	initialForm: AccountForm
	isSaving: boolean
	saveError: string | null
	onSave: (form: AccountForm) => Promise<void>
	onConvert: () => void
	onArchive: () => void
	onRestore: () => void
	openDeals: number | null
}

export function IdentitySection({
	customer,
	extras,
	defaultTerms,
	initialForm,
	isSaving,
	saveError,
	onSave,
	onConvert,
	onArchive,
	onRestore,
	openDeals,
}: IdentitySectionProps) {
	const [editing, setEditing] = useState(false)
	const [confirm, setConfirm] = useState<'convert' | 'archive' | null>(null)
	const status = accountStatusOf(customer.status)
	const kind = kindOf(customer, extras)
	const owner = findPerson(extras.ownerId)
	const professional = kind === 'professionnel'
	const registration = (customer.registration_number ?? '').trim()
	return (
		<RubriqueLayout
			title={TAB_INFO.identite.label}
			description={TAB_INFO.identite.description}
			actions={
				<RequirePermission permission="MANAGE_CUSTOMERS">
					<CustomerEditDialog
						open={editing}
						onOpenChange={setEditing}
						name={customer.name}
						initial={initialForm}
						defaultTerms={defaultTerms}
						isPending={isSaving}
						error={saveError}
						onSubmit={async (form) => {
							await onSave(form)
							setEditing(false)
						}}
						trigger={
							<Button variant="outline" size="sm">
								<Pencil aria-hidden /> Modifier
							</Button>
						}
					/>
					{status === 'prospect' && (
						<Button
							variant="outline"
							size="sm"
							onClick={() => setConfirm('convert')}
						>
							<UserCheck aria-hidden /> Convertir en client
						</Button>
					)}
					{status === 'archived' ? (
						<Button variant="outline" size="sm" onClick={onRestore}>
							<ArchiveRestore aria-hidden /> Réactiver
						</Button>
					) : (
						<Button
							variant="outline"
							size="sm"
							onClick={() => setConfirm('archive')}
						>
							<Archive aria-hidden /> Archiver
						</Button>
					)}
				</RequirePermission>
			}
		>
			<DetailCard title={professional ? 'Société' : 'Particulier'}>
				<DetailList>
					<DetailRow label="Type">{ACCOUNT_KIND_LABELS[kind]}</DetailRow>
					{professional && (
						<DetailRow label="SIRET">
							{registration ? (
								<span className="tabular">{formatSiret(registration)}</span>
							) : (
								<Dash />
							)}
						</DetailRow>
					)}
					<DetailRow label="N° de TVA">
						{extras.vatNumber ? (
							<span className="tabular">{extras.vatNumber}</span>
						) : (
							<Dash />
						)}
					</DetailRow>
					<DetailRow label="E-mail">
						{customer.email ? (
							<a
								href={`mailto:${customer.email}`}
								className="rounded-sm text-primary outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
							>
								{customer.email}
							</a>
						) : (
							<Dash />
						)}
					</DetailRow>
					<DetailRow label="Téléphone">
						{customer.phone ? (
							<span className="tabular">{customer.phone}</span>
						) : (
							<Dash />
						)}
					</DetailRow>
					<DetailRow label="Site web">
						{extras.website ? (
							<a
								href={websiteHref(extras.website)}
								target="_blank"
								rel="noreferrer"
								className="inline-flex items-center gap-1.5 rounded-sm text-primary outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
							>
								{extras.website}
								<ExternalLink className="size-3.5" aria-hidden />
								<span className="sr-only">(nouvel onglet)</span>
							</a>
						) : (
							<Dash />
						)}
					</DetailRow>
				</DetailList>
			</DetailCard>

			<DetailCard title="Suivi commercial">
				<DetailList>
					<DetailRow label="Responsable">
						{owner ? (
							<span className="inline-flex items-center gap-2">
								<PersonAvatar person={owner} />
								{owner.name}
							</span>
						) : (
							<Dash />
						)}
					</DetailRow>
					<DetailRow label="Source">
						{extras.source ? LEAD_SOURCE_LABELS[extras.source] : <Dash />}
					</DetailRow>
					<DetailRow label="Délai de paiement">
						{extras.paymentTermsDays !== null ? (
							`${extras.paymentTermsDays} jours`
						) : (
							<span>
								{defaultTerms} jours{' '}
								<span className="text-muted-foreground">
									(délai des réglages)
								</span>
							</span>
						)}
					</DetailRow>
					<DetailRow label="Créé le">
						{formatDate(customer.created_at)}
					</DetailRow>
					{extras.convertedAt && (
						<DetailRow label="Converti en client le">
							{formatDate(extras.convertedAt)}
						</DetailRow>
					)}
					{extras.archivedAt && (
						<DetailRow label="Archivé le">
							{formatDate(extras.archivedAt)}
						</DetailRow>
					)}
				</DetailList>
			</DetailCard>

			<DetailCard title="Paiements Stripe">
				<DetailList>
					<DetailRow label="Identifiant client Stripe">
						{extras.stripeCustomerId ? (
							<span className="font-mono">{extras.stripeCustomerId}</span>
						) : (
							<Dash />
						)}
					</DetailRow>
				</DetailList>
				<p className="mt-3 text-[12px] text-muted-foreground">
					Sert à rattacher automatiquement les paiements reçus par webhook.
					{extras.stripeCustomerId
						? ''
						: ' Laissé vide, il est renseigné au premier paiement rapproché par l’e-mail d’un contact.'}
				</p>
			</DetailCard>

			<DetailCard title="Notes">
				{extras.notes ? (
					<p className="whitespace-pre-wrap text-[13px]">{extras.notes}</p>
				) : (
					<p className="text-[13px] text-muted-foreground">
						Aucune note. Utilise « Modifier » pour en ajouter.
					</p>
				)}
			</DetailCard>

			<ConfirmDialog
				open={confirm === 'convert'}
				onOpenChange={(open) => !open && setConfirm(null)}
				title="Convertir en client"
				description={`${customer.name} passera de prospect à client. La date de conversion sera enregistrée à aujourd’hui.`}
				confirmLabel="Convertir en client"
				onConfirm={() => {
					onConvert()
					setConfirm(null)
				}}
			/>
			<ConfirmDialog
				open={confirm === 'archive'}
				onOpenChange={(open) => !open && setConfirm(null)}
				title={`Archiver ${customer.name} ?`}
				description={`${openDealsSentence(openDeals)} Ses devis, factures et paiements sont conservés, et tu pourras réactiver le compte à tout moment.`}
				confirmLabel="Archiver"
				destructive
				onConfirm={() => {
					onArchive()
					setConfirm(null)
				}}
			/>
		</RubriqueLayout>
	)
}
