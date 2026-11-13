import { X } from 'lucide-react'
import { useId } from 'react'
import { FormField } from '#/components/form-field'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	type DoeForm,
	PARTY_LABELS,
	type PartyForm,
	type PartyRole,
} from '#/pages/doe/types'
import { EditorCard } from '#/pages/doe/ui/editor-card'
import { ImagePicker } from '#/pages/doe/ui/image-picker'

const MAX_REVISION_LENGTH = 8

interface DoeInfoTabProps {
	form: DoeForm
	fileUrls: Record<string, string | undefined>
	isUploadingLogo: PartyRole | null
	isUploadingCover: boolean
	onFieldChange: (patch: Partial<Omit<DoeForm, 'parties' | 'sections'>>) => void
	onPartyChange: (role: PartyRole, patch: Partial<PartyForm>) => void
	onLogoUpload: (role: PartyRole, file: File) => void
	onCoverPhotoUpload: (file: File) => void
}

function ImageThumb({
	url,
	hasFile,
	alt,
	empty,
	className,
}: {
	url: string | undefined
	hasFile: boolean
	alt: string
	empty: string
	className: string
}) {
	if (!hasFile)
		return <span className="text-sm text-muted-foreground">{empty}</span>
	if (!url)
		return <span className="text-sm text-muted-foreground">Fichier envoyé</span>
	return <img src={url} alt={alt} className={className} />
}

export function DoeInfoTab({
	form,
	fileUrls,
	isUploadingLogo,
	isUploadingCover,
	onFieldChange,
	onPartyChange,
	onLogoUpload,
	onCoverPhotoUpload,
}: DoeInfoTabProps) {
	const base = useId()
	const id = (field: string) => `${base}-${field}`
	const revisionLength = form.revision.trim().length
	const revisionInvalid =
		revisionLength < 1 || revisionLength > MAX_REVISION_LENGTH

	return (
		<div className="@container grid gap-6">
			<EditorCard
				title="Informations"
				description="Le chantier tel qu’il apparaît sur la page de garde."
			>
				<div className="grid gap-4 p-5 @xl:grid-cols-2">
					<FormField id={id('title')} label="Titre du chantier" required>
						<Input
							id={id('title')}
							value={form.title}
							aria-invalid={form.title.trim() === ''}
							onChange={(event) => onFieldChange({ title: event.target.value })}
						/>
					</FormField>
					<FormField id={id('place')} label="Lieu d’intervention">
						<Input
							id={id('place')}
							value={form.place}
							onChange={(event) => onFieldChange({ place: event.target.value })}
						/>
					</FormField>
					<FormField id={id('lot')} label="N° de lot">
						<Input
							id={id('lot')}
							value={form.lot}
							onChange={(event) => onFieldChange({ lot: event.target.value })}
						/>
					</FormField>
					<FormField id={id('issued-on')} label="Date" required>
						<Input
							id={id('issued-on')}
							type="date"
							value={form.issuedOn}
							aria-invalid={form.issuedOn === ''}
							onChange={(event) =>
								onFieldChange({ issuedOn: event.target.value })
							}
						/>
					</FormField>
					<FormField id={id('author')} label="Rédacteur">
						<Input
							id={id('author')}
							value={form.author}
							onChange={(event) =>
								onFieldChange({ author: event.target.value })
							}
						/>
					</FormField>
					<FormField id={id('revision')} label="Indice" required>
						<Input
							id={id('revision')}
							value={form.revision}
							maxLength={MAX_REVISION_LENGTH}
							aria-invalid={revisionInvalid}
							aria-describedby={id('revision-hint')}
							onChange={(event) =>
								onFieldChange({ revision: event.target.value })
							}
						/>
						<p
							id={id('revision-hint')}
							className="text-[12px] text-muted-foreground"
						>
							1 à {MAX_REVISION_LENGTH} caractères.
						</p>
					</FormField>
				</div>
			</EditorCard>

			<EditorCard
				title="Photo de couverture"
				description="Affichée sur la page de garde si le style du DOE l’autorise."
			>
				<div className="grid gap-2 p-5">
					<div className="flex flex-wrap items-center gap-3">
						<ImageThumb
							url={
								form.coverPhotoKey ? fileUrls[form.coverPhotoKey] : undefined
							}
							hasFile={form.coverPhotoKey !== null}
							alt="Couverture du DOE"
							empty="Aucune photo"
							className="h-20 max-w-40 rounded border object-cover"
						/>
						<RequirePermission permission="MANAGE_PLANNING">
							<ImagePicker
								label="Choisir une photo"
								busy={isUploadingCover}
								onPick={onCoverPhotoUpload}
							/>
							{form.coverPhotoKey && (
								<Button
									type="button"
									variant="ghost"
									size="sm"
									aria-label="Retirer la photo de couverture"
									onClick={() => onFieldChange({ coverPhotoKey: null })}
								>
									<X aria-hidden />
									Retirer
								</Button>
							)}
						</RequirePermission>
					</div>
				</div>
			</EditorCard>

			<EditorCard
				title="Intervenants"
				description="Un intervenant sans nom n’est pas imprimé dans le DOE."
			>
				<div className="grid gap-4 p-5 @xl:grid-cols-2 @4xl:grid-cols-3">
					{form.parties.map((party) => (
						<PartyBlock
							key={party.role}
							party={party}
							logoUrl={party.logoKey ? fileUrls[party.logoKey] : undefined}
							busy={isUploadingLogo === party.role}
							onPatch={(patch) => onPartyChange(party.role, patch)}
							onPickLogo={(file) => onLogoUpload(party.role, file)}
						/>
					))}
				</div>
			</EditorCard>
		</div>
	)
}

function PartyBlock({
	party,
	logoUrl,
	busy,
	onPatch,
	onPickLogo,
}: {
	party: PartyForm
	logoUrl: string | undefined
	busy: boolean
	onPatch: (patch: Partial<PartyForm>) => void
	onPickLogo: (file: File) => void
}) {
	const label = PARTY_LABELS[party.role]
	const base = useId()
	const id = (field: string) => `${base}-${field}`
	return (
		<fieldset className="grid min-w-0 content-start gap-3 rounded-lg border p-4">
			<legend className="px-1 font-medium text-sm">{label}</legend>
			<FormField id={id('name')} label={`Nom (${label})`}>
				<Input
					id={id('name')}
					value={party.name}
					onChange={(event) => onPatch({ name: event.target.value })}
				/>
			</FormField>
			<FormField id={id('address')} label={`Adresse (${label})`}>
				<textarea
					id={id('address')}
					rows={3}
					value={party.address}
					onChange={(event) => onPatch({ address: event.target.value })}
					className="w-full resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
				/>
			</FormField>
			<FormField id={id('phone')} label={`Téléphone (${label})`}>
				<Input
					id={id('phone')}
					type="tel"
					value={party.phone}
					onChange={(event) => onPatch({ phone: event.target.value })}
				/>
			</FormField>
			<FormField id={id('email')} label={`E-mail (${label})`}>
				<Input
					id={id('email')}
					type="email"
					value={party.email}
					onChange={(event) => onPatch({ email: event.target.value })}
				/>
			</FormField>
			<>
				<FormField id={id('contact-name')} label="Contact : nom">
					<Input
						id={id('contact-name')}
						value={party.contactName}
						onChange={(event) => onPatch({ contactName: event.target.value })}
					/>
				</FormField>
				<FormField id={id('contact-role')} label="Contact : fonction">
					<Input
						id={id('contact-role')}
						value={party.contactRole}
						onChange={(event) => onPatch({ contactRole: event.target.value })}
					/>
				</FormField>
				<FormField id={id('contact-email')} label="Contact : e-mail">
					<Input
						id={id('contact-email')}
						type="email"
						value={party.contactEmail}
						onChange={(event) => onPatch({ contactEmail: event.target.value })}
					/>
				</FormField>
			</>
			<div className="grid gap-2">
				<span className="font-medium text-sm leading-none">
					Logo{' '}
					<span className="font-normal text-muted-foreground">
						(facultatif)
					</span>
				</span>
				<div className="flex flex-wrap items-center gap-2">
					<ImageThumb
						url={logoUrl}
						hasFile={party.logoKey !== null}
						alt={`Logo ${label}`}
						empty="Aucun logo"
						className="h-10 max-w-24 rounded border object-contain"
					/>
					<RequirePermission permission="MANAGE_PLANNING">
						<ImagePicker
							label="Choisir un logo"
							ariaLabel={`Choisir un logo (${label})`}
							busy={busy}
							onPick={onPickLogo}
						/>
						{party.logoKey && (
							<Button
								type="button"
								variant="ghost"
								size="sm"
								aria-label={`Retirer le logo (${label})`}
								onClick={() => onPatch({ logoKey: null })}
							>
								<X aria-hidden />
								Retirer
							</Button>
						)}
					</RequirePermission>
				</div>
			</div>
		</fieldset>
	)
}
